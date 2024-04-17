// @file      :  config.rs
// @author    :  fumenglin
// @time      :  2024/4/2 15:17
// @describe  :  配置文件
use serde::{Serialize, Deserialize};
use clap::Parser;
use urlencoding::decode;
use url::{Url};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub server: Server,
    pub tree: Option<Node>,
    pub local: LocalDbClient,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalDbClient {
    pub es: String,
    pub kafka: KafkaClient,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KafkaClient {
    pub brokers: String,
    pub topic: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub name: String,
    pub addr: String,
    pub next: Option<Vec<Option<Node>>>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Point {
    pub name: String,
    pub addr: String,
}

impl Point {
    pub fn new(name: String, addr: String) -> Self {
        Point {
            name,
            addr,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    pub ip: String,
    pub port: String,
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(path)?;
    let scrape_config: Config = serde_yaml::from_reader(f)?;
    return Ok(scrape_config);
}

pub fn load_config_from_str(content: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let scrape_config: Config = serde_yaml::from_str(content)?;
    return Ok(scrape_config);
}


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Input {
    /// 配置文件的路径
    #[arg(long)]
    pub naocs: Option<String>,

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NacosRegister {
    pub username: String,
    pub password: String,
    pub addr: String,
    pub namespace: String,
}

pub fn parse_nacos(url: &str) -> Result<NacosRegister, url::ParseError> {
    let issue_url = Url::parse(url)?;
    let password = issue_url.password().unwrap();
    let password = decode(password).unwrap();
    let host = issue_url.host().unwrap();
    let namespace = issue_url.path().to_string();
    let namespace = namespace.replace("/", "");
    return Ok(NacosRegister {
        username: issue_url.username().to_string(),
        password: password.to_string(),
        addr: format!("{}:{}", host.to_string(), issue_url.port().unwrap().to_string()),
        namespace,
    });
}