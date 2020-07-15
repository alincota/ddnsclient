mod mythic_beasts;

use crate::config;

use std::error::Error;
use serde::{Serialize, Deserialize};

// impl Provider {
    // pub fn from_config(&self, c: config::Configuration) -> Provider {
        // let creds: config::Credentials = c
            // .credentials
            // .into_iter()
            // .filter(|cred| cred.provider == self.name)
            // .collect();

        // Provider {
            // name: self.name.clone(),
            // credentials: Some(creds),
        // }
    // }
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub host: String,
    pub ttl: u32,
    pub r#type: String,
    pub data: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mx_priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub srv_priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub srv_weight: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub srv_port: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sshfp_algorithm: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sshfp_type: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caa_flags: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caa_property: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlsa_usage: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlsa_selector: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlsa_matching: Option<u32>,

    #[serde(skip)]
    pub _template: Option<bool>,
}

pub fn init_provider(name: &str) -> Box<dyn Provider> {
    match name {
        "mythic-beasts" => mythic_beasts::MythicBeasts::new(),
        _ => unimplemented!(),
    }
}

pub fn get_provider_credentials(provider: &Box<dyn Provider>, c: config::Configuration) -> config::Credentials {
    let creds: config::Credentials = c
    .credentials
    .into_iter()
    .filter(|cred| cred.provider == provider.get_name())
    .collect();

    creds
}



pub trait Provider: std::fmt::Debug {
    fn get_name(&self) -> String;
    fn set_credentials(&mut self, c: config::Credentials);
    fn get_credential(&self, zone: String, host: Option<String>, r#type: Option<String>) -> Result<(&str, Option<&str>), Box<dyn Error>>;
    fn dynamic_dns(&self, zone: Option<String>, host: Option<String>) -> Result<(), Box<dyn Error>>;
    // fn search();
    // fn update();
    // fn delete();
}


