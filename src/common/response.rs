// @file      :  response.rs
// @author    :  fumenglin
// @time      :  2024/3/26 18:05
// @describe  :


use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::common::error;
use axum::{body::Body, response::Response};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseVo<T> {
    pub status: bool,
    pub code: u8,
    pub msg: String,
    pub body: Option<T>,
}

impl<T> ResponseVo<T>
    where
        T: Serialize + DeserializeOwned + Clone,
{
    pub fn from_result(arg: &Result<T, error::Error>) -> Self {
        if arg.is_ok() {
            Self {
                code: 0,
                msg: "成功".to_string(),
                status: true,
                body: arg.clone().ok(),
            }
        } else {
            Self {
                code: 1,
                msg: arg.clone().err().unwrap().to_string(),
                status: false,
                body: None,
            }
        }
    }

    pub fn from(arg: &T) -> Self {
        Self {
            code: 0,
            msg: "成功".to_string(),
            status: true,
            body: Some(arg.clone()),
        }
    }

    pub fn from_error(arg: &error::Error) -> Self {
        Self {
            code: 1,
            msg: arg.to_string(),
            status: false,
            body: None,
        }
    }

    pub fn from_error_info(info: &str) -> Self {
        Self {
            code: 1,
            msg: info.to_string(),
            status: false,
            body: None,
        }
    }

    pub fn resp_json(&self) -> Response<Body> {
        Response::builder()
            .extension(|| {})
            .header("Access-Control-Allow-Origin", "*")
            .header("Cache-Control", "no-cache")
            .header("Content-Type", "application/json")
            .body(Body::from(self.to_string()))
            .unwrap()
    }

    pub fn resp_json_forward_info(&self,path:&str) ->Response<Body>{
        Response::builder()
            .extension(|| {})
            .header("Access-Control-Allow-Origin", "*")
            .header("Cache-Control", "no-cache")
            .header("Content-Type", "application/json")
            .header("Forward-Back",format!("{}",path))
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}

impl<T> ToString for ResponseVo<T>
    where
        T: Serialize + DeserializeOwned + Clone,
{
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}