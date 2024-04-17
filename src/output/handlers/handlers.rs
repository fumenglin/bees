// @file      :  handlers.rs
// @author    :  fumenglin
// @time      :  2024/4/2 19:15
// @describe  :


use crate::common::error::Error;
use serde_json::Value;
use crate::output::handlers::{to_es::save::ElasticClient};

#[derive(Debug, Clone)]
pub struct OutputClient {
    pub kfk_brokers: Vec<String>,
    pub kfk_topic: String,
    pub es_client: ElasticClient,
}


pub trait LocalSave {
    //async fn send(&mut self, body: Value) -> Result<(), Error>;
    fn get_source(&self, body: Value) -> Result<Value, Error> {
        if let Some(body) = body.as_object().and_then(|body| body.get("_source")) {
            Ok(body.clone())
        } else {
            return Err(Error::E("no body".to_string()));
        }
    }

    fn get_index(&self, body: Value) -> Result<String, Error> {
        if let Some(index) = body.as_object().and_then(|body| body.get("_index")) {
            let index = index.to_string();
            Ok(index)
        } else {
            return Err(Error::E("no index".to_string()));
        }
    }
}