// @file      :  forward.rs
// @author    :  fumenglin
// @time      :  2024/3/26 17:31
// @describe  :

use axum::Router;
use axum::routing::post;
//use kafka::client::metadata::Broker;
use tracing::error;
use crate::common::error::Error;
use crate::forward::local::{local_handlers};
use crate::output::handlers::{handlers::OutputClient,
                              to_es::save::{elasticsearch_client, ElasticClient}};
use crate::common::config::{LocalDbClient};

pub fn router(local: LocalDbClient) -> Router {
    let brokers = vec![local.clone().kafka.brokers];
    let state_client = get_client_init(local.clone().es.as_str(), brokers, local.clone().kafka.topic);
    match state_client {
        Ok(state_client) => {
            return Router::new()
                .route("/local", post(local_handlers)).with_state(state_client);
        }
        Err(err) => {
            error!("本地连接出错： {:#?}",err);
            panic!("本地连接出错： {:#?}", err)
        }
    }
}


fn get_client_init(es_url: &str, brokers: Vec<String>, topic: String) -> Result<OutputClient, Error> {
    let es_client = elasticsearch_client(es_url);
    if es_client.is_err() {
        return Err(Error::E(es_client.err().unwrap().to_string()));
    }
    let es_clt = ElasticClient {
        client: es_client.unwrap()
    };
    Ok(OutputClient {
        es_client: es_clt,
        kfk_brokers: brokers,
        kfk_topic: topic,
    })
}