use super::{DnsProvider};
use crate::config;

#[derive(Debug)]
pub struct MythicBeasts<'a> {
    name: &'a str,
    credentials: Option<config::Credentials>,
}

impl<'a> MythicBeasts<'a> {
    pub fn new() -> MythicBeasts<'a> {
        MythicBeasts {
            name: "mythic-beasts",
            credentials: None,
        }
    }
}

impl<'p> DnsProvider<'p> for MythicBeasts<'p> {
    const API_URL: &'static str = "https://api.mythic-beasts.com/dns/v2";

    fn get_name(&self) -> &'p str {
        self.name
    }

    // fn set_credentials<'a, C: DnsProvider<'a> + std::fmt::Debug>(&self, creds: config::Credentials) -> C {
        // MythicBeasts {
            // name: self.name,
            // credentials: Some(creds),
        // }
    // }

    fn dynamic_dns(&self) {
        println!("DNSProvider trait...ddns to {}", MythicBeasts::API_URL);
    }
}
