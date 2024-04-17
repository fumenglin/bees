// @file      :  main.rs
// @author    :  fumenglin
// @time      :  2024/3/26 15:46
// @describe  :  数据同步，多个服务直接的数据转发

use time;
use tracing::{error, info};
use tracing_subscriber::fmt::{time::LocalTime};
use bees::common::{icon::print_bees_icon, config::Input, config::parse_nacos};
use bees::core::core::core_start;
use clap::Parser;
use std::env;
use std::{result::Result, error::Error};
use bees::common::config::{Config, load_config_from_str};
use bees::common::nacos::{get_cfg, Nacos, RegisterService};
use bees::common::consts::{APP_NAME, NACOS_HOST_ENV};

fn get_nacos_var() -> String {
    let args = Input::parse();
    if let Some(nacos_var) = args.naocs {
        return nacos_var;
    };
    let env_name = env::var(NACOS_HOST_ENV);
    match env_name {
        Ok(nacos) => {
            return nacos;
        }
        Err(err) => {
            panic!("获取nacos参数地址数据错误：{:#?}", err)
        }
    }
}

#[tokio::main]
async fn main() {
    let time_format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .expect("format string should be valid!");
    let timer = LocalTime::new(time_format);
    let subscriber = tracing_subscriber::fmt()
        //.with_max_level(tracing::Level::DEBUG)
        .with_timer(timer);
    subscriber.init();


    print_bees_icon();

    if env::consts::OS == "windows" {
        let nacos_path = "nacos:wst@nacos@10.13.15.89:8848";//"nacos:wst@nacos@172.17.228.84:6648/4e62e868-c2c2-4d4f-8990-cea9d98431a9";
        env::set_var(NACOS_HOST_ENV, nacos_path);
    };

    let nacos_path = get_nacos_var();
    let nacos_path = format!("http://{}", nacos_path);
    let nacos_info = parse_nacos(nacos_path.as_str()).unwrap();
    info!("nacos配置参数{},解析 {:#?}", nacos_path, nacos_info);
    let nacos = Nacos::new(nacos_info.addr.as_str(),
                           nacos_info.namespace.as_str(),
                           nacos_info.username.as_str(),
                           nacos_info.password.as_str());
    // let config_service = nacos.config_service().unwrap();
    // let cfg = get_config(config_service, APP_NAME).await.unwrap();
    let cfg = get_cfg(&nacos_info).await.unwrap();
    let cfgs = load_config_from_str(cfg.as_str());
    //info!("bees 配置数据为{:#?}", cfgs);

    // let config_path = "D:\\code\\rust\\bees\\bees.yaml";
    // info!("当前服务的配置文件路径为： {}",config_path);


    info!("启动数据、服务转发服务.");

    //let cfg = load_config(config_path);
    match cfgs {
        Ok(ref cf) => {
            // //服务注册
            let service = RegisterService {
                name: APP_NAME.to_string(),
                ip: cf.clone().server.ip,
                port: cf.clone().server.port.parse::<i32>().unwrap(),
            };
            let rg = nacos.naming_service_and_register(&service);
            if rg.is_err() {
                panic!("bees服务注册失败：{:#?}", rg.err().unwrap());
            }
            // let rg = instance_register(&nacos_info, &service).await;
            // if rg.is_err() {
            //     panic!("bees服务注册失败：{:#?}", rg.err().unwrap());
            // }
        }
        Err(e) => {
            error!("读取配置文件错误：{:#?}",e);
            panic!("读取配置文件错误：{:#?}", e)
        }
    }

    let cofg = cfgs.unwrap();


    if let Err(e) = start(&cofg, &nacos).await {
        error!("{:#?}",e);
        panic!("不好了，程序启动崩溃了")
    }
    info!("数据、服务转发服务启动完成.");
}

async fn start(cfg: &Config, out_service: &Nacos) -> Result<(), Box<dyn Error>> {
    core_start(cfg, out_service).await?;
    Ok(())
}