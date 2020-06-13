mod mythic_beasts;

use crate::config;
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

pub fn init_provider(name: &str) -> (impl DnsProvider + std::fmt::Debug) {
    match name {
        "mythic-beasts" => mythic_beasts::MythicBeasts::new(),
        _ => unimplemented!(),
    }
}

pub fn get_provider_credentials<'a, P: DnsProvider<'a>>(provider: &'a P, c: config::Configuration) -> config::Credentials {
    let creds: config::Credentials = c
    .credentials
    .into_iter()
    .filter(|cred| cred.provider == provider.get_name())
    .collect();

    creds
}

pub trait DnsProvider<'p> {
    const API_URL: &'static str;

    fn get_name(&self) -> &'p str;
    // fn set_credentials<'a, P: DnsProvider<'a> + std::fmt::Debug>(&self, creds: config::Credentials) -> P;
    fn dynamic_dns(&self);
    // fn search();
    // fn update();
    // fn delete();
}


