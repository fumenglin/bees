// @file      :  save.rs
// @author    :  fumenglin
// @time      :  2024/4/2 19:12
// @describe  :  直接存储到kafka的topic

use kafka::producer::{Producer, Record, RequiredAcks};
use serde_json::Value;
use std::time::Duration;
use crate::common::error::Error;
use crate::output::handlers::handlers::LocalSave;

// #[derive(Debug, Clone)]
pub struct KafkaProduce {
    pub producer: Producer,
    pub topic: String,
}


pub fn kakfa_produce_init(brokers: Vec<String>) -> Result<Producer, Error> {
    let producer = Producer::from_hosts(brokers)
        .with_ack_timeout(Duration::from_secs(2))
        .with_required_acks(RequiredAcks::One)
        .create();
    match producer {
        Ok(producer) => Ok(producer),
        Err(err) => Err(Error::E(err.to_string()))
    }
}

impl LocalSave for KafkaProduce {}

impl KafkaProduce {
    pub fn new(producer: Producer, topic: String) -> Self {
        KafkaProduce {
            producer,
            topic,
        }
    }
    pub(crate) async fn send(&mut self, data: Value) -> Result<(), Error> {
        //获取source
        let data = self.get_source(data.clone())?;
        let binding = data.clone().to_string();
        let byte = binding.as_bytes();
        let record = Record::from_value(self.topic.as_str(), byte);
        let resp = self.producer.send(&record);
        match resp {
            Ok(_resp) => Ok(()),
            Err(err) => Err(Error::E(err.to_string())),
        }
    }

    fn get_source(&self, body: Value) -> Result<Value, Error> {
        let index = self.get_index(body.clone())?;
        if let Some(body) = body.as_object().and_then(|body| body.get("_source")) {
            if let Some( body_map) = body.as_object() {
                body_map.clone()["table"] = index.parse().unwrap();
                return Ok(Value::from(body_map.clone()));
            }
            return Ok(body.clone());
        } else {
            return Err(Error::E("no body".to_string()));
        }
    }
}