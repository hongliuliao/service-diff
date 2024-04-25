
use std::thread::JoinHandle;

use crossbeam::channel::{bounded, Receiver, Sender};
use crate::replay_log_reader::ReplayLog;
use crate::result_diff::ResultDiff;
use crate::http_client::HttpClient;

#[derive(Clone)]
pub struct RequestServiceConf {
    pub old_url: String,
    pub new_url: String,
    pub concurrent_num: i32,
    pub timeout_ms: u64,
    pub queue_size: usize,
    pub request_method: String,
}

pub const GET_METHOD: &'static str = "GET";
pub const POST_METHOD: &'static str = "POST";

impl RequestServiceConf {
    pub fn new() -> RequestServiceConf {
        RequestServiceConf {
            old_url: String::new(),
            new_url: String::new(),
            concurrent_num: 2,
            timeout_ms: 500,
            queue_size: 200,
            request_method: String::from(GET_METHOD),
        }
    }
}

use std::time::SystemTime;
use std::sync::atomic::AtomicUsize;

pub struct StatInfo {
    pub succ_cnt: AtomicUsize,
    pub fail_cnt: AtomicUsize,
    pub start_ts: std::time::SystemTime,
    pub end_ts: std::time::SystemTime,
}

impl StatInfo {
    pub fn new() -> std::sync::Arc<StatInfo> {
        std::sync::Arc::new(StatInfo{
            succ_cnt: AtomicUsize::new(0),
            fail_cnt: AtomicUsize::new(0),
            start_ts: SystemTime::now(),
            end_ts: SystemTime::now(),
        })
    }
}

pub struct RequestService {
    service_conf: RequestServiceConf,
    log_sender: Sender<ReplayLog>,
    log_receiver: Receiver<ReplayLog>,
    pub join_handles: Vec<std::thread::JoinHandle<()>>,
    pub stat_info: std::sync::Arc<StatInfo>,
}

impl RequestService {

    pub fn new<'a>(service_conf: RequestServiceConf) -> Result<RequestService, Box<dyn std::error::Error>> {    
        if service_conf.new_url.is_empty() {
            Err("input new_url is empty")?
        }
        if service_conf.old_url.is_empty() {
            Err("input old_url is empty")?
        }
        let (tx, rx) = bounded::<ReplayLog>(service_conf.queue_size);
        let mut service = RequestService{
            service_conf,
            log_sender: tx, 
            log_receiver: rx,
            join_handles: Vec::new(),
            stat_info: StatInfo::new(),
        };
        for _ in 0..service.service_conf.concurrent_num {
            let tx_clone = service.log_receiver.clone();
            let service_conf = service.service_conf.clone();
            let stat_info = service.stat_info.clone();
            
            let join_handle: JoinHandle<()>  = std::thread::spawn(move || {
                loop {
                    let msg_ret: Result<ReplayLog, crossbeam::channel::RecvError> = tx_clone.recv();
                    if msg_ret.is_err() {
                        log::info!("get msg from channel fail, err:{}", msg_ret.err().unwrap());
                        break;
                    }
                    let log: ReplayLog = msg_ret.ok().unwrap();
                    let handle_ret = RequestService::handle_replay_log(&log, &service_conf);
                    if handle_ret.is_err() {
                        log::error!("handle replay log fail, err:{}", handle_ret.err().unwrap());
                        let _ = &stat_info.fail_cnt.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        continue;
                    }
                    let _ = &stat_info.succ_cnt.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            });
            service.join_handles.push(join_handle);
        }
        Ok(service)
    }

    pub fn handle_replay_log(log: &ReplayLog, service_conf: &RequestServiceConf) -> Result<bool, Box<dyn std::error::Error>> {
        log::info!("handle_replay_log log: {}, old_url:{}, new_url:{}",
            log.data, service_conf.old_url, service_conf.new_url);
        let mut old_rsp = String::new();
        let mut new_rsp = String::new();
        if service_conf.request_method == GET_METHOD {
            let old_url = service_conf.old_url.clone() + "?" + log.data.as_str();
            let new_url = service_conf.new_url.clone() + "?"  + log.data.as_str();
            old_rsp = HttpClient::get(old_url.as_str(), service_conf.timeout_ms)?;
            log::info!("send request succ, old_url:{}, rsp:{old_rsp}", service_conf.old_url);
            new_rsp = HttpClient::get(new_url.as_str(), service_conf.timeout_ms)?;
            log::info!("send request succ, new_url:{}, rsp:{new_rsp}", service_conf.new_url);
        } else if service_conf.request_method == POST_METHOD {
            old_rsp = HttpClient::post_json(&service_conf.old_url, &log.data, service_conf.timeout_ms)?;
            new_rsp = HttpClient::post_json(&service_conf.new_url, &log.data, service_conf.timeout_ms)?;
        } else {
            Err(format!("unkonw request method :{}", service_conf.request_method))?
        }
       
        ResultDiff::diff(service_conf, old_rsp, new_rsp);
        return Ok(true)
    }

    pub fn send_to_channel(&self, logs: Vec<ReplayLog>) -> Result<(), Box<dyn std::error::Error>> {
        for log in logs {
            let send_ret = self.log_sender.send(log);
            if send_ret.is_err() {
                Err("send to channel fail")?
            }
        }
        return Ok(())
    }

}
