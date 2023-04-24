use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub UDPPort: i32,
    pub DataSize: usize,
    pub ErrorRate: i32,
    pub LostRate: i32,
    pub SWSize: i32,
    pub InitSeqNo: i32,
    pub Timeout: i32,
    pub Where: String,
    pub WhichPort: i32,
    pub FileToSend:String
}

impl Config {
    pub fn read(config_name: &str) -> Config {
        let config_json = std::fs::read_to_string(config_name).expect("读取配置文件错误");
        let config: Config = serde_json::from_str(&config_json).expect("将配置文件反序列化错误");
        return config;
    }
}