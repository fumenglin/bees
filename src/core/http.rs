// @file      :  http.rs
// @author    :  fumenglin
// @time      :  2024/3/26 17:09
// @describe  :  http服务引擎

use std::{result::Result, error::Error};
use tower_http::cors::{Any, CorsLayer};
use std::time::Duration;
use axum::{Router};
use axum::{middleware::{self}};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use tokio::net::TcpListener;
use tracing::log::info;
use crate::common::config::{LocalDbClient, Node};
use crate::common::nacos::Nacos;
use crate::forward::core::{forward_middleware, ForwardClient};

use crate::forward::{forward, api};


#[derive(Debug, Clone)]
pub struct Http {
    pub ip: String,
    pub port: String,
    pub tree: Option<Node>,
    pub local: LocalDbClient,
    pub out_service: Nacos,
}


impl Http {
    pub fn new(ip: String, port: String, tree: Option<Node>, local: LocalDbClient, out_service: Nacos) -> Self {
        Http {
            ip,
            port,
            tree,
            local,
            out_service,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        let addr = format!("0.0.0.0:{}", self.port);
        let server_addr = format!("{}:{}", self.ip, self.port);
        let state = ForwardClient::new(hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new()).build(HttpConnector::new()),
                                       server_addr.clone(),
                                       self.tree.clone(),
                                       self.out_service.clone());
        _ = state.update_router_map();
        let forward_path = state.router_map.clone();
        info!("当前转发服务的地址为： {}",server_addr.clone());
        let cors = self.cors();
        let ping = Router::new().route("/", get(iam)).with_state(server_addr.clone());

        let app = Router::new()
            //数据转发服务
            .nest("/forward", forward::router(self.local.clone()))
            //相关api
            .nest("/api", api::router(forward_path))
            .nest("/ping", ping)

            .layer(middleware::from_fn_with_state(state, forward_middleware))
            .layer(cors);

        let listener = TcpListener::bind(addr.clone().as_str()).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }

    fn cors(&self) -> CorsLayer {
        CorsLayer::new()
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_origin(Any)
            .max_age(Duration::from_secs(360))
    }
}


async fn iam(State(addr): State<String>) -> impl IntoResponse {
    format!("Hello,I`m {}!", addr)
}