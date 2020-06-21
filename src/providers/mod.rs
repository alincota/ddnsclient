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
    fn dynamic_dns(&self, hostname: &str);
    // fn search();
    // fn update();
    // fn delete();
}


