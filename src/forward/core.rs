// @file      :  core.rs
// @author    :  fumenglin
// @time      :  2024/4/1 16:32
// @describe  :  数据转发的，数据流转发


use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Uri};
use axum::response::{IntoResponse};
use std::string::ToString;
use std::sync::{Arc, Mutex};
use axum::middleware::Next;
use std::collections::HashMap;
use std::ops::Index;
use hyper_util::client::legacy::connect::HttpConnector;
use tracing::log::{error, info, warn};


use crate::common::config::{Node, Point};
use crate::common::error::Error;
use crate::common::nacos::Nacos;
use crate::common::response::ResponseVo;


const FORWARD_PATH: &str = "Forward-Path";
const FORWARD_BACK: &str = "Forward-Back";

const DESTINATION_SERVICE: &str = "Destination-Service";


//每个节点的链路，从root往下
pub type RouterMap = Arc<Mutex<HashMap<String, Vec<Point>>>>;

#[derive(Clone, Debug)]
pub struct ForwardClient {
    pub fwd_client: hyper_util::client::legacy::Client<HttpConnector, Body>,
    pub host: String,
    pub tree: Option<Node>,
    pub router_map: RouterMap,
    pub out_service: Nacos,
}

impl ForwardClient {
    pub fn new(fwd_client: hyper_util::client::legacy::Client<HttpConnector, Body>, host: String, tree: Option<Node>, out_service: Nacos) -> Self {
        ForwardClient {
            fwd_client,
            host,
            tree,
            router_map: Arc::new(Mutex::new(HashMap::new())),
            out_service,
        }
    }

    pub fn update_router_map(&self) -> Result<(), Error> {
        //从上面的treed通过深度遍历，获取每个接口的链路
        let init_path = Vec::new();

        if let Some(root_node) = self.tree.clone() {
            let mut is_root = false;
            if self.host == root_node.addr {
                info!("当前为master平台");
                is_root = true
            } else {
                info!("当前为下级平台");
            }
            println!("{:30}  {:100}", "name", "path");
            self.dfs(is_root, root_node, init_path);
        } else {
            warn!("当前服务没有配置服务节点.");
        }
        Ok(())
    }

    //深度优先算法寻找路径
    fn dfs(&self, is_root: bool, node: Node, mut path: Vec<Point>) {
        let point = Point::new(node.name.clone(), node.addr.clone());
        if is_root {
            path.insert(path.len(), point.clone());
        } else {
            path.insert(0, point.clone());
        }
        let mut path_str = String::new();
        for pp in path.clone() {
            if path_str.clone() == "" {
                path_str += format!("{}:{}", pp.addr.clone().as_str(), pp.name.clone().as_str()).as_str();
            } else {
                path_str += format!("-> {}:{}", pp.addr.clone().as_str(), pp.name.clone().as_str()).as_str();
            }
        }
        println!("{:30}  {:100}", node.name.to_string(), path_str.clone());
        self.router_map.lock().unwrap().insert(node.addr.to_string(), path.clone());
        if node.next.is_none() {
            return;
        }
        if let Some(next) = node.next {
            for nxt in next {
                if let Some(nd) = nxt {
                    self.dfs(is_root, nd, path.clone())
                }
            }
        }
    }
}


fn forward_path_is_exist(headers: &HeaderMap) -> bool {
    if let Some(forward_path) = headers.get(FORWARD_PATH) {
        if let Some(forward_path_str) = forward_path.to_str().ok() {
            if forward_path_str.len() > 0 {
                return true;
            }
        }
    }
    return false;
}

pub async fn forward_middleware(
    State(client): State<ForwardClient>,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let f_exist = forward_path_is_exist(request.headers());
    let dest = get_destination_service(request.headers());
    if !f_exist && dest.is_some() {
        //不存在的时候，则添加
        if let Some(path_) = client.router_map.lock().unwrap().get(client.host.clone().as_str()) {
            let mut nodes = Vec::new();
            for node in path_ {
                nodes.push(node.addr.clone())
            }
            let forward_path = nodes.join(",");
            if forward_path.len() > 0 {
                request.headers_mut().insert(FORWARD_PATH, forward_path.clone().as_str().parse().unwrap());
            }
        }
    }

    //如果Destination-Service不为空的时候，当当前host和最后的一个host想等的时候，再在后面通过nacos查询服务添加到最后
    if let Some(dest_srv) = request.headers().get(DESTINATION_SERVICE) {
        if let Some(dest) = dest_srv.to_str().ok() {
            if dest.len() > 0 {
                let forward_path = get_forward_path(request.headers());
                if current_host_is_end(forward_path.as_str(), client.host.as_str()) {
                    //通过nacos查询当前服务
                    //let nacos_info = client.out_service.clone().client_info();
                    //let out_service = client.out_service.clone().get_instances_new(&nacos_info,dest).await;
                    let out_service = client.out_service.clone().get_instances(dest).await;
                    match out_service {
                        Ok(out_service) => {
                            for ins in out_service {
                                if ins.enabled && ins.healthy {
                                    let out_service_addr = format!("{}:{}", ins.ip, ins.port.to_string());
                                    //let out_service_addr ="127.0.0.1:3205".to_string();
                                    if out_service_addr.as_str() == client.host.clone().as_str() {
                                        //如果目标服务和本服务一样，则跳过，防止无限循环
                                        continue;
                                    }
                                    let forward_path = format!("{},{}", forward_path, out_service_addr);
                                    request.headers_mut().insert(FORWARD_PATH, forward_path.as_str().parse().unwrap());
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            error!("通过nacos获取目标服务出现错误：{:#?}",err);
                            let msg = String::from(format!("目标服务地址获取失败{}，{:#?}", dest, err));
                            return ResponseVo::<String>::from_error(&Error::from(msg.as_str())).resp_json();
                        }
                    }
                }
            }
        }
    }

    //需要转发的url
    if let Some(forward_path) = request.headers().get(FORWARD_PATH) {
        if let Some(forward_path_str) = forward_path.to_str().ok() {
            info!("当前转发的整体路径为：{}，当前服务为：{}",forward_path_str,client.host.as_str());
            if let Some(next_host) = get_next_host(forward_path_str, client.host.as_str()) {
                return forward(client, request, next_host.clone().as_str()).await.into_response();
            }
        }
    }
    // let url = request.uri().path();
    // if url == "/ioc/sync4"{
    //     let body = request.into_body();
    //     println!("{:#?}",body);
    //     let msg = String::from(format!("访问失败"));
    //     ResponseVo::<String>::from_error(&Error::from(msg.as_str())).resp_json_forward_info(format!(":err").as_str())
    // }else {
    //
    // }
    let response = next.run(request).await;

    response.into_response()
}


fn get_forward_path(headers: &HeaderMap) -> String {
    if let Some(forward_path) = headers.get(FORWARD_PATH) {
        if let Some(forward_path_str) = forward_path.to_str().ok() {
            return forward_path_str.to_string();
        }
    }
    return "".to_string();
}

fn get_destination_service(headers: &HeaderMap) -> Option<String> {
    if let Some(dest_srv) = headers.get(DESTINATION_SERVICE) {
        if let Some(dest) = dest_srv.to_str().ok() {
            if dest.len() > 0 {
                return Some(dest.to_string());
            }
        }
    }
    return None;
}

fn current_host_is_end(forward_path: &str, current_host: &str) -> bool {
    let sp: Vec<&str> = forward_path.split(",").collect();
    if sp.len() > 0 {
        let last_host = sp.index(sp.len() - 1);
        if last_host.to_string().as_str() == current_host {
            return true;
        }
    }
    return false;
}


fn get_next_host(forward_path: &str, current_host: &str) -> Option<String> {
    let sp: Vec<&str> = forward_path.split(",").collect();
    let l = sp.len();
    let mut index = 0;
    for ss in sp.clone() {
        if ss == current_host {
            break;
        }
        index += 1;
    }
    if index + 1 < l {
        let next = sp.clone()[index + 1];
        return Some(next.to_string());
    }
    return None;
}


pub fn get_forward_next_uri(next_host: &str, path_query: &str) -> String {
    if path_query != "/" {
        return format!("http://{}{}", next_host, path_query);
    } else {
        return format!("http://{}", next_host);
    }
}

pub async fn forward(client: ForwardClient, mut req: Request, next_host: &str) -> impl IntoResponse {
    let clients = client.fwd_client;
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = get_forward_next_uri(next_host, path_query);

    info!("当前请求转发到下一个：method: {} , addr: {}",req.method().to_string(),uri.clone());

    *req.uri_mut() = Uri::try_from(uri.clone()).unwrap();

    let response = clients
        .request(req)
        .await;
    match response {
        Ok(mut res) => {
            if let Some(forward_back_str) = res.headers().get(FORWARD_BACK) {
                let forward_back_str = format!("{},{}:{}", forward_back_str.to_str().unwrap_or(""), next_host, "ok");
                res.headers_mut().insert(FORWARD_BACK, HeaderValue::from_str(forward_back_str.as_str()).unwrap());
            } else {
                let forward_back_str = format!("{}:{}", next_host, "ok");
                res.headers_mut().insert(FORWARD_BACK, HeaderValue::from_str(forward_back_str.as_str()).unwrap());
            }
            let result = res.into_response();
            result
        }
        Err(e) => {
            error!("转发的目的服务返回错误：{:#?}",e);
            let msg = String::from(format!("{}访问失败", next_host));
            ResponseVo::<String>::from_error(&Error::from(msg.as_str())).resp_json_forward_info(format!("{}:err", next_host).as_str())
        }
    }
}



