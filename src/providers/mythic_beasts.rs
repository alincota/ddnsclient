use super::{Provider, Record, ProviderError, ProviderErrorKind, Result};
use crate::config;

use serde::{Serialize, Deserialize};
use clap::{ArgMatches};

const API_URL: &str = "https://api.mythic-beasts.com/dns/v2";

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
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
        // We either have one authentication credential configured -OR- user has used user-pass approach
        if let Some(credential) = &self.credentials {
            if credential.len() == 1 {
                return Ok(credential[0].clone());
            }
        }

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

        let credential: config::Credentials = self.credentials
            .as_ref()
            .unwrap_or(&vec![])
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

    fn build_api_endpoint(app: &ArgMatches, filter: Option<&str>) -> String {
        let mut endpoint = format!("{}/zones", API_URL);

        if app.is_present("zone") {
            let zone = app.value_of("zone").unwrap();
            endpoint.push_str(&format!("/{}/records", zone));
        }

        if app.is_present("host") {
            let host = app.value_of("host").unwrap();
            endpoint.push_str(&format!("/{}", host));
        }

        if app.is_present("type") {
            let r#type = app.value_of("type").unwrap();
            endpoint.push_str(&format!("/{}", r#type));
        }

        if let Some(f) = filter {
            endpoint.push_str(&format!("?{}", f));
        }

        endpoint
    }
}


impl Provider for MythicBeasts {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_credentials(&mut self, c: config::Credentials) {
        self.credentials = Some(c);
    }

    // TODO: support subdomain.domain.tld format as well. CHG config to allow credentials to
    // map to FQDNs (this will help support NoIP that uses FQDN instead of zones and hosts)
    fn dynamic_dns(&self, argm: &ArgMatches) -> Result<bool>{
        if !argm.is_present("zone") {
            log::error!("Zone missing for DDNS!");
            return Ok(true);
        }

        if !argm.is_present("host") {
            log::error!("Host missing for DDNS!");
            return Ok(true);
        }

        let zone = argm.value_of("zone").unwrap();
        let host = argm.value_of("host").unwrap();
        let endpoint = format!("{}/zones/{}/dynamic/{}", API_URL, zone, host);

        let credentials = self.get_credential(zone, Some(host), None)?;

        let response = reqwest::blocking::Client::new()
            .put(&endpoint)
            .basic_auth(credentials.user, Some(credentials.pass))
            .send()?;

        let text = response.text()?;
        log::trace!("Received response: {}", &text);

        let result: ApiResponse = serde_json::from_str(&text)?;

        if let Some(e) = result.error {
            return Err(ProviderError::new(ProviderErrorKind::DnsApiError)
                .msg(format!("Unable to use DDNS feature. Reason: {}", e)));
        }

        log::info!("{}", result.message.unwrap());

        Ok(true)
    }

    fn search(&self, argm: &ArgMatches) -> Result<Option<Vec<Record>>> {
        let url = MythicBeasts::build_api_endpoint(argm, None);

        let zone = argm.value_of("zone").expect("DNS search requires at least a zone to start from");
        let host = argm.value_of("host");
        let rtype = argm.value_of("type");
        let credentials = self.get_credential(zone, host, rtype)?;

        let response = reqwest::blocking::Client::new()
            .get(&url)
            .basic_auth(credentials.user, Some(credentials.pass))
            .send()?;

        let text = response.text()?;
        log::trace!("Received response: {}", &text);

        let result: ApiResponse = serde_json::from_str(&text)?;
        log::trace!("{:#?}", result);

        if let Some(e) = result.error {
            return Err(ProviderError::new(ProviderErrorKind::DnsApiError)
                .msg(format!("Unable to get search results. Reason: {}", e)));
        }

        Ok(result.records)
    }

    fn delete(&self, argm: &ArgMatches) -> Result<bool> {
        let url = MythicBeasts::build_api_endpoint(argm, Some("exclude-generated=true&exclude-template=true"));
        let zone = argm.value_of("zone").expect("Deleting DNS record(s) requires at least a zone to be provided");
        let host = argm.value_of("host");
        let rtype = argm.value_of("type");
        let credentials = self.get_credential(zone, host, rtype)?;

        let response = reqwest::blocking::Client::new()
            .delete(&url)
            .basic_auth(credentials.user, Some(credentials.pass))
            .send()?;

        let response_status = response.status();
        let text = response.text()?;
        log::trace!("Received response: {}", &text);

        let result: ApiResponse = serde_json::from_str(&text)?;
        log::trace!("{:#?}", result);

        if response_status.is_client_error() {
            if response_status == reqwest::StatusCode::BAD_REQUEST {
                if let Some(e) = result.errors {
                    return Err(ProviderError::new(ProviderErrorKind::DnsApiError)
                        .msg(format!("Unable to delete selected record(s). Reasons: \n - {}", e.join("\n - "))));
                }
            }

            if let Some(e) = result.error {
                return Err(ProviderError::new(ProviderErrorKind::DnsApiError)
                    .msg(format!("Unable to delete selected record(s). Reason: {}", e)));
            }
        }

        if let Some(r) = result.records_removed {
            log::info!("Deleted {} record(s)", r);
            return Ok(true);
        }

        Ok(true)
    }

    fn update(&self, argm: &ArgMatches, records: &Vec<Record>) -> Result<bool> {
        let mut recs = std::collections::HashMap::new();
        recs.insert("records", records);

        let url = MythicBeasts::build_api_endpoint(argm, Some("exclude-generated=true&exclude-template=true"));

        let zone = argm.value_of("zone").expect("Updating DNS record(s) requires at least the zone to be specified!");
        let host = argm.value_of("host");
        let rtype = argm.value_of("type");
        let credentials = self.get_credential(zone, host, rtype)?;

        let response = reqwest::blocking::Client::new()
            .put(&url)
            .basic_auth(credentials.user, Some(credentials.pass))
            .json(&recs)
            .send()?;

        let response_status = response.status();
        let text = response.text()?;
        log::trace!("Received response: {}", &text);

        let result: ApiResponse = serde_json::from_str(&text)?;
        log::trace!("{:#?}", result);

        if response_status.is_client_error() {
            if response_status == reqwest::StatusCode::BAD_REQUEST {
                if let Some(e) = result.errors {
                    return Err(ProviderError::new(ProviderErrorKind::DnsApiError)
                        .msg(format!("Unable to update selected record(s). Reasons: \n - {}", e.join("\n - "))));
                }
            }

            if let Some(e) = result.error {
                return Err(ProviderError::new(ProviderErrorKind::DnsApiError)
                    .msg(format!("Unable to update selected record(s). Reason: {}", e)));
            }
        }

        let records_added: u32 = match result.records_added {
            Some(n) => n,
            None => 0,
        };

        let records_removed: u32 = match result.records_removed {
            Some(n) => n,
            None => 0,
        };

        log::info!("Updated record(s)!");
        log::debug!("Added {} record(s). Removed {} record(s)", records_added, records_removed);

        Ok(true)
    }
}
