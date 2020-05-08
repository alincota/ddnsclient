extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

// use std::error::Error;
// use std::net::IpAddr; // used for get_my_public_ip()
// use std::str::FromStr; // used for get_my_public_ip()
use std::process;

use clap::{Arg, App, SubCommand, crate_version, crate_authors};


fn main() {
    let app = App::new("DDNS Client")
        .version(crate_version!())
        .author(crate_authors!())
        .after_help("SUPPORTED PROVIDERS:\n - Mythic Beasts (https://www.mythic-beasts.com/support/api/dnsv2)")
        .arg(Arg::with_name("verbosity")
            .short("v")
            .global(true)
            .multiple(true)
            .help("Set the level of verbosity for logging. Use -v for errors up to -vvvv for more detailed information."))

        .arg(Arg::with_name("zone")
            .long("zone")
            // .required(true)
            .takes_value(true)
            .number_of_values(1)
            .help("The name of the zone")
        )
        .arg(Arg::with_name("host")
            .long("host")
            // .required(true)
            .takes_value(true)
            .number_of_values(1)
            .help("The hostname to create or update. This can be either the hostname within the zone (e.g. www) or a fully-qualified host name (e.g. www.example.com)")
        )
        .arg(Arg::with_name("username")
            .short("u")
            .long("username")
            .required(true)
            .takes_value(true)
            .number_of_values(1)
            .help("Authentication username")
        )
        .arg(Arg::with_name("password")
            .short("p")
            .long("password")
            .takes_value(true)
            .number_of_values(1)
            .help("Authentication password")
        )

        .subcommand(SubCommand::with_name("ddns")
            .about("Create or update an A or AAAA record with the specified hostname, with the data set to the IP address of the client using the API.")
            .arg(Arg::with_name("host")
                .required(true)
                .takes_value(true)
                .value_name("HOSTNAME")
                .help("The fully-qualified host name")
            )
        )
        // .subcommand(SubCommand::with_name("")
        // )

        .get_matches();


    let log_level = match app.occurrences_of("verbosity") {
            0 => log::Level::Error,
            1 => log::Level::Warn,
            2 => log::Level::Info,
            3 => log::Level::Debug,
            _ => log::Level::Trace,
    };
    simple_logger::init_with_level(log_level).expect("Unable to initialise the logger!");

    // First check the IP of the client
    /* let ip = get_my_public_ip(); */
    // if let Err(e) = ip {
        // log::error!("{}", e);
        // process::exit(exitcode::UNAVAILABLE);
    // }
    /* let ip = ip.unwrap(); */

    // let zone = app.value_of("zone").unwrap();
    // let host = app.value_of("host").unwrap();

    // todo: Read external config file so we can set credentials based on future providers
    let username = app.value_of("username").unwrap();
    let password = app.value_of("password");

    match app.subcommand() {
        ("ddns", Some(ddns)) => {
            let host = ddns.value_of("host").unwrap();

            if let Err(e) = mythic_beasts::dynamic_dns(host, username, password) {
                log::error!("{}", e);
                process::exit(exitcode::UNAVAILABLE);
            }

            return ();
        },
        _ => (),
    }
}

mod mythic_beasts {
    use std::error::Error;
    // use std::net::IpAddr; // used for get_my_public_ip()
    use serde::{Serialize, Deserialize};

    const API_URL: &str = "https://api.mythic-beasts.com/beta/dns";

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ApiResponse {
        pub error: Option<String>,
        pub message: Option<String>,
    }

    // todo: add structures for different responses/objects such as zones, record types and so on.
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Record {
        pub host: String,
        pub ttl: u32,
        pub r#type: String,
        pub data: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct MxRecord {
        #[serde(flatten)]
        pub record: Record,
        pub mx_priority: u32,

        // todo: check https://serde.rs/attr-flatten.html (capture additional fields) for an alternative approach to the MX record type
    }

    pub fn dynamic_dns(host: &str, username: &str, password: Option<&str>) -> Result<(), Box<dyn Error>> {
        let endpoint = format!("{}/dynamic/{}", API_URL, host);
        let response = reqwest::blocking::Client::new()
            .put(&endpoint)
            .basic_auth(username, password)
            .send()?;

        let text = response.text()?;
        log::trace!("Received response: {}", &text);

        let result: ApiResponse = serde_json::from_str(&text)?;

        if let Some(e) = result.error {
            return Err(format!("Unable to use ddns feature. Reason: {}", e).into());
        }
        
        log::info!("{}", result.message.unwrap());

        Ok(())
    }

    #[allow(dead_code)]
    fn get_record_types(username: &str, password: Option<&str>) -> Result<(), Box<dyn Error>> {
        let endpoint = format!("{}/record-types", API_URL);
        let response = reqwest::blocking::Client::new()
            .get(&endpoint)
            .basic_auth(username, password)
            .send()?;

        let text = response.text()?;
        log::trace!("Received response: {}", &text);

        // Received response: {"A":["host","ttl","type","data"],"AAAA":["host","ttl","type","data"],"ANAME":["host","ttl","type","data"],"CAA":["host","ttl","type","caa_flags","caa_tag","data"],"CNAME":["host","ttl","type","data"],"DNAME":["host","ttl","type","data"],"MX":["host","ttl","type","mx_priority","data"],"NS":["host","ttl","type","data"],"SRV":["host","ttl","type","host","srv_priority","srv_weight","srv_port","data"],"SSHFP":["host","ttl","type","sshfp_algorithm","sshfp_type","data"],"TLSA":["host","ttl","type","tlsa_usage","tlsa_selector","tlsa_matching","data"],"TXT":["host","ttl","type","data"]}

        Ok(())
    }

    #[allow(dead_code)]
    fn get_zones() -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}


/* fn get_my_public_ip() -> Result<IpAddr, Box<dyn Error>> { */
    // let opendns_ip = reqwest::blocking::get("https://diagnostic.opendns.com/myip")?.text()?;
    // let ipify_ip = reqwest::blocking::get("https://api6.ipify.org")?.text()?;

    // let opendns_ip = IpAddr::from_str(&opendns_ip)?;
    // let ipify_ip = IpAddr::from_str(&ipify_ip)?;

    // log::debug!("OpenDNS IPAddr: {:?}", opendns_ip);
    // log::debug!("IPify IP: {:?}", ipify_ip);

    // if opendns_ip != ipify_ip {
        // return Err("OpenDNS and IPify services responded with different IPs.".into());
    // }

    // Ok(opendns_ip)
/* } */