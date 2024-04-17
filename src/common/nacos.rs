// @file      :  nacos.rs
// @author    :  fumenglin
// @time      :  2024/4/7 10:53
// @describe  :  通过nacos获取配置或许相应的服务

use nacos_sdk::api::config::{ConfigService, ConfigServiceBuilder};
use nacos_sdk::api::constants;
use nacos_sdk::api::naming::{NamingService, NamingServiceBuilder, ServiceInstance};
use nacos_sdk::api::props::ClientProps;
use nacos_sdk::api::error::Result;
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use nacos_sdk::api::error::Error;
use crate::common::config::NacosRegister;
use crate::common::consts::APP_NAME;

#[derive(Debug, Clone)]
pub struct Nacos {
    pub client: ClientProps,
    client_info: NacosRegister,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterService {
    pub name: String,
    pub ip: String,
    pub port: i32,
}

impl Nacos {
    pub fn new(nacos_addr: &str, namespace: &str, username: &str, password: &str) -> Self {
        Nacos {
            client: ClientProps::new()
                .server_addr(nacos_addr)
                .app_name("bees")
                .namespace(namespace)
                .auth_username(username)
                .auth_password(password),
            client_info: NacosRegister {
                username: username.to_string(),
                password: password.to_string(),
                addr: nacos_addr.to_string(),
                namespace: namespace.to_string(),
            },
        }
    }
    pub fn client_info(&self) -> NacosRegister {
        return self.client_info.clone();
    }
    pub fn naming_service_and_register(&self, service: &RegisterService) -> Result<()> {
        let naming_service = NamingServiceBuilder::new(self.client.clone())
            .enable_auth_plugin_http()
            .build().unwrap();
        let service_instance = ServiceInstance {
            ip: service.ip.clone(),
            port: service.port.clone(),
            ..Default::default()
        };
        let res = naming_service.register_instance(service.name.clone(), Some(constants::DEFAULT_GROUP.to_string()), service_instance.clone());
        return res;
    }

    pub fn config_service(&self) -> Result<impl ConfigService> {
        let config_service = ConfigServiceBuilder::new(self.client.clone())
            .enable_auth_plugin_http()
            .build();
        return config_service;
    }

    pub async fn get_instances(&self, name: &str) -> Result<Vec<ServiceInstance>> {
        let naming_service = NamingServiceBuilder::new(self.client.clone())
            .enable_auth_plugin_http()
            .build().unwrap();
        let register_instance_ret = naming_service.get_all_instances(
            name.to_string(),
            Some(constants::DEFAULT_GROUP.to_string()),
            Vec::default(),
            false,
        );
        return register_instance_ret;
    }
    pub async fn get_instance_2(&self, name: &str) -> Result<Vec<ServiceInstance>> {
        //println!("{}",self.client_info.password.clone());
        //let pw = self.client_info.password.clone().replace("%40", "@");
        let client_props = ClientProps::new()
            .server_addr(self.client_info.addr.clone())
            // .remote_grpc_port(9838)
            // Attention! "public" is "", it is recommended to customize the namespace with clear meaning.
            .namespace(self.client_info.namespace.clone())
            .app_name(APP_NAME)
            .auth_username(self.client_info.username.clone())
            .auth_password(self.client_info.password.clone())
            ;

        // ----------  Naming  -------------
        let naming_service = NamingServiceBuilder::new(client_props)
            .enable_auth_plugin_http()
            .build()?;

        let register_instance_ret = naming_service.get_all_instances(
            name.to_string(),
            Some(constants::DEFAULT_GROUP.to_string()),
            Vec::default(),
            false,
        );
        return register_instance_ret;
    }

    pub async fn get_instances_new(&self, name: &str) -> Result<Vec<ServiceInstance>> {
        // let client = ClientProps::new()
        //     .server_addr(cli.addr.clone())
        //     .app_name("bees")
        //     .namespace(cli.namespace.clone())
        //     .auth_username(cli.username.clone())
        //     .auth_password("wst@nacos");
        let client_props = ClientProps::new()
            .server_addr("10.13.15.89:8848")
            // .remote_grpc_port(9838)
            // Attention! "public" is "", it is recommended to customize the namespace with clear meaning.
            .namespace("")
            .app_name("simple_app")
            .auth_username("nacos") // TODO You can choose not to enable auth
            .auth_password("wst@nacos") // TODO You can choose not to enable auth
            ;
        let naming_service = NamingServiceBuilder::new(client_props.clone())
            //.enable_auth_plugin_http()
            .build()?;
        let register_instance_ret = naming_service.get_all_instances(
            name.to_string(),
            Some(constants::DEFAULT_GROUP.to_string()),
            Vec::default(),
            false,
        );
        return register_instance_ret;
    }
}


pub async fn get_config(config_service: impl ConfigService, name: &str) -> StdResult<String, Error> {
    let name = format!("{}.yaml", name);
    let config_resp = config_service.get_config(name.to_string(), constants::DEFAULT_GROUP.to_string());
    match config_resp {
        Ok(config_resp) => {
            tracing::info!("get the config {:#?}", config_resp.content());
            return Ok(config_resp.content().to_string());
        }
        Err(err) => {
            tracing::error!("get the config {:?}", err);
            return Err(err);
        }
    }
}


pub async fn get_cfg(client: &NacosRegister) -> StdResult<String, Error> {
    //let pw = client.password.replace("%40", "@");
    let client_props = ClientProps::new()
        .server_addr(client.addr.clone())
        .namespace(client.namespace.clone())
        .app_name(APP_NAME)
        .auth_username(client.username.clone())
        .auth_password(client.password.clone());


    // ----------  Config  -------------
    let config_service = ConfigServiceBuilder::new(client_props.clone())
        .enable_auth_plugin_http()
        .build()?;
    let name = format!("{}.yaml", APP_NAME);
    let config_resp = config_service.get_config(name, "DEFAULT_GROUP".to_string());
    match config_resp {
        Ok(config_resp) => {
            tracing::info!("get the config {:#?}", config_resp);
            return Ok(config_resp.content().to_string());
        }
        Err(err) => {
            tracing::error!("get the config {:?}", err);
            return Err(err);
        }
    }
}


pub async fn instance_register(client: &NacosRegister, ins: &RegisterService) -> Result<()> {
    //let pw = client.password.clone().replace("%40", "@");
    let client_props = ClientProps::new()
        .server_addr(client.addr.clone())
        .namespace(client.namespace.clone())
        .app_name(APP_NAME)
        .auth_username(client.username.clone())
        .auth_password(client.password.clone())
        ;

    // ----------  Naming  -------------
    let naming_service = NamingServiceBuilder::new(client_props)
        .enable_auth_plugin_http()
        .build()?;

    let service_instance1 = ServiceInstance {
        ip: ins.ip.clone(),
        port: ins.port.clone(),
        ..Default::default()
    };
    let res = naming_service.register_instance(APP_NAME.to_string(),
                                               Some(constants::DEFAULT_GROUP.to_string()),
                                               service_instance1.clone());
    return res;
}