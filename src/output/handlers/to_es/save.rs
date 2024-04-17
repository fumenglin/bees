// @file      :  save.rs
// @author    :  fumenglin
// @time      :  2024/4/2 19:12
// @describe  :  存储到es

use elasticsearch::{Error as EsError, Elasticsearch, IndexParts};
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use serde_json::{ Value};
use url::Url;
use crate::output::handlers::handlers::LocalSave;
use crate::common::error::Error;

#[derive(Debug, Clone)]
pub struct ElasticClient {
    pub client: Elasticsearch,
}


pub fn elasticsearch_client(url: &str) -> Result<Elasticsearch, EsError> {
    let url_parse = Url::parse(url)?;
    let conn_pool = SingleNodeConnectionPool::new(url_parse);
    let transport = TransportBuilder::new(conn_pool).disable_proxy().build()?;
    Ok(Elasticsearch::new(transport))
}


impl LocalSave for ElasticClient {
}

impl ElasticClient {
    pub fn new(client: Elasticsearch) -> Self {
        ElasticClient {
            client
        }
    }


    fn get_id(&self, body: Value) -> Result<String, Error> {
        if let Some(id) = body.as_object().and_then(|body| body.get("_id")) {
            let id = id.to_string();
            Ok(id)
        } else {
            return Err(Error::E("no id".to_string()));
        }
    }
    pub async fn send(&mut self, body: Value) -> Result<(), Error> {
        //获取index，id
        let index = self.get_index(body.clone())?;
        let index = index.replace("\"", "").replace("\\", "");
        let id = self.get_id(body.clone())?;
        let id = id.replace("\"", "").replace("\\", "");
        let body = self.get_source(body.clone())?;
        let response = self.client.index(IndexParts::IndexId(index.as_str(), id.as_str()))
            .body(body)
            .send().await;
        match response {
            Ok(response) => {
                if response.status_code().is_success() {
                    Ok(())
                } else {
                    Err(Error::E(response.status_code().to_string()))
                }
            }
            Err(err) => Err(Error::E(err.to_string())),
        }
    }
}