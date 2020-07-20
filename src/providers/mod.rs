mod mythic_beasts;

use crate::config;

use std::fmt;
use std::error;
use serde::{Serialize, Deserialize};
use clap::{ArgMatches};

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



pub trait Provider: fmt::Debug {
    fn get_name(&self) -> String;
    fn set_credentials(&mut self, c: config::Credentials);

    /// Dynamic DNS capability
    fn dynamic_dns(&self, argm: &ArgMatches) -> Result<bool>;

    /// Search for DNS record(s) via the API
    fn search(&self, argm: &ArgMatches) -> Result<Option<Vec<Record>>>;

    /// Update DNS record(s)
    fn update(&self, argm: &ArgMatches, records: &Vec<Record>) -> Result<bool>;

    /// Delete DNS record(s)
    fn delete(&self, argm: &ArgMatches) -> Result<bool>;
}



/// An error which can be returned when working with a DNS provider feature/capability.
///
/// This error is used as the error type for all trait object functions as well as on any of
/// the helper functions.
#[derive(Debug)]
pub struct ProviderError {
    kind: ProviderErrorKind,
    // TODO: consider converting this into a &str as we should know all error messages sizes
    message: Option<String>,
}

/// Enum to store various types of errors that can cause the application to fail.
#[derive(Debug)]
pub enum ProviderErrorKind {
    CredentialNotFound,
    ReqwestFail(reqwest::Error),
    SerdeJsonError(serde_json::Error),
    DnsApiError,
}

type Result<T> = std::result::Result<T, ProviderError>;

impl ProviderError {
    pub fn new(kind: ProviderErrorKind) -> Self {
        ProviderError {
            kind,
            message: None,
        }
    }

    /// Add (optionally) a different error message
    /// * `msg` - New message string
    fn msg(mut self, msg: String) -> Self {
        self.message = Some(msg);
        self
    }

    #[doc(hidden)]
    fn __get_default_message(&self) -> String {
        match &self.kind {
            ProviderErrorKind::CredentialNotFound => String::from("Unable to find credential!"),
            ProviderErrorKind::ReqwestFail(e) => format!("Reqwest: {}", e.to_string()),
            ProviderErrorKind::SerdeJsonError(e) => format!("Serde-JSON: {}", e.to_string()),
            ProviderErrorKind::DnsApiError => String::from("Received API error!"),
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match &self.message {
            None => self.__get_default_message(),
            Some(m) => m.to_string(),
        };

        write!(f, "{}", msg)
    }
}

impl error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.kind {
            ProviderErrorKind::ReqwestFail(e) => Some(e),
            ProviderErrorKind::SerdeJsonError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> ProviderError {
        ProviderError::new(ProviderErrorKind::ReqwestFail(err))
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> ProviderError {
        ProviderError::new(ProviderErrorKind::SerdeJsonError(err))
    }
}
