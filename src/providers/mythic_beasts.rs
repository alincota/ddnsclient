use super::{Provider};
use crate::config;

#[derive(Debug)]
pub struct MythicBeasts {
    name: String,
    API_URL: String,
    credentials: Option<config::Credentials>,
}

impl MythicBeasts {
    pub fn new() -> Box<dyn Provider> {
        Box::new(MythicBeasts {
            API_URL: String::from("https://api.mythic-beasts.com/dns/v2"),
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

    fn dynamic_dns(&self, host: &str) {
        println!("Provider trait...ddns to {}", self.API_URL);
    }
}
