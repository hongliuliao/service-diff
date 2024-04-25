mod replay_log_reader;
mod request_service;
mod result_diff;
mod http_client;

use replay_log_reader::ReplayLogReader;
use request_service::{RequestService, RequestServiceConf};
use structopt::StructOpt;
use simple_logger::SimpleLogger;

use crate::request_service::StatInfo;

#[derive(Clone, StructOpt)]
pub struct DiffConf {
    #[structopt(long)]
    pub old_url: String,
    #[structopt(long)]
    pub new_url: String,
    #[structopt(short = "c", default_value = "10")]
    pub concurrent_num: i32,
    #[structopt(default_value = "1000")]
    pub timeout_ms: u64,
    #[structopt(default_value = "100")]
    pub queue_size: usize,
    #[structopt(long)]
    pub request_method: String,
    #[structopt(long)]
    pub replay_log_path: String,
}

fn set_service_conf_val(service_conf: &mut RequestServiceConf, conf: DiffConf) {
    service_conf.old_url = conf.old_url;
    service_conf.new_url = conf.new_url;
    service_conf.request_method = conf.request_method;
    service_conf.concurrent_num = conf.concurrent_num;
    service_conf.timeout_ms = conf.timeout_ms;
    service_conf.queue_size = conf.queue_size;
}

fn read_and_diff(conf: DiffConf) -> Result<bool, Box<dyn std::error::Error>> {
    let mut reader = ReplayLogReader::new(&conf.replay_log_path)?;
    let mut service_conf = RequestServiceConf::new();
    set_service_conf_val(&mut service_conf, conf);
    
    let join_handlers: Vec<std::thread::JoinHandle<()>>;
    let stat_info: std::sync::Arc<StatInfo>;
    {
        let request_service = RequestService::new(service_conf)?;
        
        loop {
            let lines = reader.read_lines(5)?;
            if lines.is_empty() {
                break;
            }
            log::info!("指定行数读取到的内容: {:?}, 行数：{}", lines, lines.len());
            request_service.send_to_channel(lines)?;
        }
        join_handlers = request_service.join_handles;
        stat_info = request_service.stat_info;
    }
    
    let handler_len = join_handlers.len();
    log::info!("start wait handle over, size:{}", join_handlers.len());
    for join_handle in join_handlers { 
        join_handle.join().unwrap();
    }
    let cost_ms = std::time::SystemTime::now().duration_since(stat_info.start_ts).unwrap().as_millis();
    log::info!("Wait handle over, cost_ms:{cost_ms}, succ_cnt:{}, fail_cnt:{}, handler size:{}",
        stat_info.succ_cnt.load(std::sync::atomic::Ordering::Relaxed),
        stat_info.fail_cnt.load(std::sync::atomic::Ordering::Relaxed), handler_len);
    Ok(true)
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::set_max_level(log::LevelFilter::Info);
    SimpleLogger::new().init().unwrap();

    let diff_conf = DiffConf::from_args();
    let ret = read_and_diff(diff_conf);
    if ret.is_err() {
        log::error!("read and find diff fail, err:{}", ret.err().unwrap());
    }
    
    Ok(())
}
