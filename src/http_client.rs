
use std::io::Read;

use curl::easy::Easy;

pub struct HttpClient {

}

impl HttpClient {

    pub fn get(url: &str, timeout_ms: u64) -> Result<String, Box<dyn std::error::Error>> {
        let mut easy = Easy::new();
        log::info!("HttpClient start get request, url:{url}");
        easy.url(&url)?;
        easy.timeout(std::time::Duration::from_millis(timeout_ms))?;
        let mut rsp = String::new();
        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                for a in data {
                    let a_char: char = a.clone() as char;
                    rsp.push(a_char);
                }
                //rsp_data.extend_from_slice(data);
                
                Ok(data.len())
            })?;
            
            transfer.perform()?;
        }
        log::info!("HttpClient get url:{url}, rsp:{rsp}");
        Ok(rsp)
    }

    pub fn post_json(url: &str, post_data: &str, timeout_ms: u64) -> Result<String, Box<dyn std::error::Error>> {
        let mut easy = Easy::new();
        log::info!("HttpClient start post request, url:{url}");
        easy.url(&url)?;
        easy.post(true)?;
        let mut list = curl::easy::List::new();
        list.append("Content-Type: application/json")?;
        easy.http_headers(list)?;
        easy.post_field_size(post_data.len() as u64)?;

        easy.timeout(std::time::Duration::from_millis(timeout_ms))?;
        let mut rsp = String::new();
        {
            let mut transfer = easy.transfer();
            transfer.read_function(|buf| {
                Ok(post_data.as_bytes().read(buf).unwrap_or(0))
            })?;
            transfer.write_function(|data| {
                for a in data {
                    let a_char: char = a.clone() as char;
                    rsp.push(a_char);
                }
                Ok(data.len())
            })?;
            
            transfer.perform()?;
        }
        log::info!("HttpClient post url:{url}, data:{post_data}, rsp:{rsp}");
        Ok(rsp)
    }
}