use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Credential {
    pub provider: String,
    pub user: String,
    pub pass: String,
    pub zone: Option<String>,
    pub host: Option<String>,
    pub r#type: Option<String>,
}

pub type Credentials = Vec<Credential>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub credentials: Credentials,
}


impl Configuration {
    // fn new() -> Configuration {
    // }

    pub fn from_path(path: &str) -> Configuration {
        let config_contents = fs::read_to_string(path).expect("Unable to read config file!");
        let config: Configuration = serde_yaml::from_str(&config_contents).unwrap();

        config
    }
}



