use crate::request_service::RequestServiceConf;

pub struct ResultDiff {

}

impl ResultDiff {

    pub fn diff(service_conf: &RequestServiceConf, old_content: String, new_content: String) -> bool {
        let is_diff = old_content != new_content;
        if is_diff {
            let old_url = service_conf.old_url.clone();
            let new_url = service_conf.new_url.clone();
            log::warn!("Find diff, old_req:{old_url}, old_rsp:{old_content},
                 new_req:{new_url}, new_content: {new_content}")
        }
        return is_diff;
    }
}