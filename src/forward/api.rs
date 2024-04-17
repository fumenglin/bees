// @file      :  api.rs
// @author    :  fumenglin
// @time      :  2024/3/26 17:43
// @describe  :  对外的api服务

use axum::extract::State;
use axum::Router;
use axum::routing::get;
use axum::response::IntoResponse;
use crate::common::response::ResponseVo;
use crate::forward::core::RouterMap;

pub fn router(forward_path: RouterMap) -> Router {
    Router::new()
        .route("/forward/path/get", get(forward_path_get)).with_state(forward_path)
}

async fn forward_path_get(State(forward_path): State<RouterMap>) -> impl IntoResponse {
    let mut result = Vec::new();
    let forward_path = forward_path.clone();
    for fwd in forward_path.lock().unwrap().values() {
        let mut fwd_str_vec = Vec::new();
        for point in fwd {
            fwd_str_vec.push(point.addr.clone())
        }
        let fwd_str = fwd_str_vec.join(",");
        result.push(fwd_str);
    }
    ResponseVo::from(&result).resp_json()
}