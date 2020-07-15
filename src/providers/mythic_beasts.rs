use super::{Provider, Record};
use crate::config;

use std::error::Error;
use serde::{Serialize, Deserialize};
use clap::{ArgMatches};

const API_URL: &str = "https://api.mythic-beasts.com/dns/v2";

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub error: Option<String>,
    pub errors: Option<Vec<String>>,
    pub message: Option<String>,
    pub records_added: Option<u32>,
    pub records_removed: Option<u32>,
    pub records: Option<Vec<Record>>,
}


#[derive(Debug)]
pub struct MythicBeasts {
    name: String,
    credentials: Option<config::Credentials>,
}

impl MythicBeasts {
    pub fn new() -> Box<dyn Provider> {
        Box::new(MythicBeasts {
            name: String::from("mythic-beasts"),
            credentials: None,
        })
    }
}


impl Provider for MythicBeasts {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_credentials(&mut self, c: config::Credentials) {
        self.credentials = Some(c);
    }

    fn get_credential(&self, zone: String, host: Option<String>, r#type: Option<String>) -> Result<(&str, Option<&str>), Box<dyn Error>> {
        // filter credentials based on zone and host
        Ok(("username...", Some("password...")))
    }

    fn dynamic_dns(&self, zone: Option<String>, host: Option<String>) -> Result<(), Box<dyn Error>> {
        let endpoint = format!("{}/zones/{}/dynamic/{}", API_URL, zone.unwrap(), host.unwrap());
        // let (username, password) = self.get_credential(zone.unwrap(), host, None);
        // let credential: (String, String) = match self.get_credential(zone.unwrap(), host, None) {}

        // let response = reqwest::blocking::Client::new()
            // .put(&endpoint)
            // .basic_auth(username, password)
            // .send()?;

        // let text = response.text()?;
        // log::trace!("Received response: {}", &text);

        // let result: ApiResponse = serde_json::from_str(&text)?;

        // if let Some(e) = result.error {
            // return Err(format!("Unable to use DDNS feature. Reason: {}", e).into());
        // }

        // log::info!("{}", result.message.unwrap());

        Ok(())
    }
}
