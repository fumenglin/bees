// @file      :  local.rs
// @author    :  fumenglin
// @time      :  2024/4/8 9:12
// @describe  :

use axum::extract::State;
use axum::{Json};
use axum::response::IntoResponse;
use tracing::info;
use crate::common::response::ResponseVo;
use crate::output::handlers::{to_kafka::save::{KafkaProduce, kakfa_produce_init}};
use crate::output::handlers::handlers::OutputClient;
use crate::common::error::Error;


pub async fn local_handlers(State(client): State<OutputClient>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    //处理数据
    //1、存人es
    //2、发送给卡夫卡
    let es_client = client.es_client;
    let es_resp = es_client.clone().send(payload.clone()).await;
    if es_resp.is_err() {
        let err_msg = format!("save to elastic err :{}", es_resp.err().unwrap().to_string());
        return ResponseVo::<()>::from_error(&Error::E(err_msg)).resp_json();
    };

    let kfk_client = kakfa_produce_init(client.kfk_brokers);
    if kfk_client.is_err() {
        let err_msg = format!("kfk_client err :{}", kfk_client.err().unwrap().to_string());
        return ResponseVo::<()>::from_error(&Error::E(err_msg)).resp_json();
    };

    let mut kfk_producer = KafkaProduce::new(kfk_client.unwrap(), client.kfk_topic);
    let kfk_resp = kfk_producer.send(payload.clone()).await;

    if kfk_resp.is_err() {
        let err_msg = format!("save to kafka err :{}", kfk_resp.err().unwrap().to_string());
        return ResponseVo::<()>::from_error(&Error::E(err_msg)).resp_json();
    };

    info!("数据存储到本地成功：{:#?}",payload.clone().to_string());
    let msg = String::from("Are you ok!");
    ResponseVo::<String>::from(&msg).resp_json()
}

// pub async fn request(request: Request)-> impl IntoResponse {
//     // println!("{:#?}",request.extract());
//     let msg = String::from("Are you ok!");
//     ResponseVo::<String>::from(&msg).resp_json()
// }

// use bytes::Bytes;
// use axum::{
//     extract::FromRequest, http::StatusCode, };
// use axum::body::Body;
// use std::task::{Context, Poll};
//
//
// struct ValidatedBody(Bytes);
//
// impl<S> FromRequest<S> for ValidatedBody
//     where
//         Bytes: FromRequest<S>,
//         S: Send + Sync,
// {
//     type Rejection = Response;
//
//     async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
//         let body = Bytes::from_request(req, state)
//             .await
//             .map_err(IntoResponse::into_response)?;
//
//         // do validation...
//
//         Ok(Self(body))
//     }
// }
//
// async fn handler(ValidatedBody(body): ValidatedBody) {
//     // ...
// }