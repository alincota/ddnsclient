use super::{Provider, Record, ProviderError, ProviderErrorKind, Result};
use crate::config;

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

    fn get_credential(&self, zone: &str, host: Option<&str>, r#type: Option<&str>) -> Result<config::Credential> {
        let credential_filter = |c: &&config::Credential| -> bool {
            let host_check = match host {
                None => c.host == None,
                Some(h) => c.host == Some(h.to_string()),
            };

            let rtype_check = match r#type {
                None => c.r#type == None,
                Some(rtype) => c.r#type == Some(rtype.to_string()),
            };

            c.zone == Some(zone.to_string()) && host_check && rtype_check
        };

        let credential: config::Credentials = self.credentials.as_ref().unwrap()
            .iter()
            .filter(credential_filter)
            // .inspect(|i| println!("item that passed the filter: {:?}", i))
            .cloned()
            .collect();

        if credential.is_empty() {
            return Err(ProviderError::new(ProviderErrorKind::CredentialNotFound));
        }

        Ok(credential[0].clone())
    }
}


impl Provider for MythicBeasts {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_credentials(&mut self, c: config::Credentials) {
        self.credentials = Some(c);
    }

    fn dynamic_dns(&self, argm: &ArgMatches) -> Result<()>{
        if !argm.is_present("zone") {
            log::error!("Zone missing for DDNS!");
            return Ok(());
        }

        if !argm.is_present("host") {
            log::error!("Host missing for DDNS!");
            return Ok(());
        }

        let zone = argm.value_of("zone").unwrap();
        let host = argm.value_of("host").unwrap();
        let endpoint = format!("{}/zones/{}/dynamic/{}", API_URL, zone, host);

        let credentials = self.get_credential(zone, Some(host), None)?;

        // let response = reqwest::blocking::Client::new()
            // .put(&endpoint)
            // .basic_auth(credentials.user, Some(credentials.pass))
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
