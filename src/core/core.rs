// @file      :  core.rs
// @author    :  fumenglin
// @time      :  2024/3/26 17:09
// @describe  :  核心启动模块


use tracing::info;
use crate::common::config::Config;
use crate::common::nacos::Nacos;
use crate::core::http::Http;

pub async fn core_start(cfg: &Config, out_service: &Nacos) -> Result<(), Box<dyn std::error::Error>> {
    let http_ser = Http::new(cfg.server.ip.clone(), cfg.server.port.clone(), cfg.tree.clone(), cfg.local.clone(), out_service.clone());
    info!("http服务启动成功,port:{}",cfg.server.ip.clone());
    http_ser.start().await?;
    Ok(())
}