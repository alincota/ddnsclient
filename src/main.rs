extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::error::Error; // used for get_my_public_ip()
use std::net::IpAddr; // used for get_my_public_ip()
use std::str::FromStr; // used for get_my_public_ip()
use std::process;

use clap::{Arg, App, SubCommand};


fn main() {
    let app = App::new("DDNS Client")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .after_help("SUPPORTED PROVIDERS:\n - Mythic Beasts (https://www.mythic-beasts.com/support/api/dnsv2)")
        .arg(Arg::with_name("verbosity")
            .short("v")
            .global(true)
            .multiple(true)
            .help("Set the level of verbosity for logging. Use -v for errors up to -vvvv for more detailed information."))
        .arg(Arg::with_name("zone")
            .global(true)
            .takes_value(true)
            .number_of_values(1)
            .help("The name of the zone")
        )
        .arg(Arg::with_name("host")
            .global(true)
            .takes_value(true)
            .number_of_values(1)
            .help("The hostname. This can be either the hostname within the zone (e.g. www) or a fully-qualified host name (e.g. www.example.com). Use @ to return records for the bare domain (apex).")
        )
        .arg(Arg::with_name("type")
            .global(true)
            .takes_value(true)
            .number_of_values(1)
            .help("The record type")
        )

        // todo: replace username and pass with the config file (yaml)
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
            .arg(Arg::with_name("hostname")
                .required(true)
                .takes_value(true)
                .value_name("HOSTNAME")
                .help("The fully-qualified host name")
            )
        )
        .subcommand(SubCommand::with_name("delete")
            .about("Deletes all records selected by the zone|host|type")
        )
        // .subcommand(SubCommand::with_name("")
        // )


        .get_matches();


    // Set up logging
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

    // todo: Read external config file so we can set credentials based on future providers
    let username = app.value_of("username").unwrap();
    let password = app.value_of("password");

    // todo: consider catching errors outside of match for  all subcommands
    match app.subcommand() {
        ("ddns", Some(ddns)) => {
            let host = ddns.value_of("hostname").unwrap();

            if let Err(e) = mythic_beasts::dynamic_dns(host, username, password) {
                log::error!("{}", e);
                process::exit(exitcode::UNAVAILABLE);
            }

            return ();
        },
        ("update", Some(_u)) => {
            unimplemented!("Update command missing implementation");
        },
        ("delete", Some(_d)) => {
            match mythic_beasts::delete(&app, username, password) {
                Ok(r) => {
                    println!("Deleted {} record(s)", r);
                    return ();
                },
                Err(e) => {
                    log::error!("{}", e);
                    process::exit(exitcode::UNAVAILABLE);
                },
            }
        },
        _ => (),
    }

    let api_endpoint = mythic_beasts::build_api_endpoint(&app);

    match mythic_beasts::search(api_endpoint, username, password) {
        Ok(records) => {
            // todo: once happy with tests, change to to_string()
            match serde_json::to_string_pretty(&records) {
                Ok(s) => {
                    println!("{}", s);
                    return ();
                },
                Err(e) => {
                    log::error!("{}", e);
                    process::exit(exitcode::UNAVAILABLE);
                }
            }
        },
        Err(e) => {
            log::error!("{}", e);
            process::exit(exitcode::UNAVAILABLE);
        }
    }
}








mod mythic_beasts {
    use std::error::Error;
    // use std::net::IpAddr; // used for get_my_public_ip()
    use serde::{Serialize, Deserialize};
    use clap::{ArgMatches};

    const API_URL: &str = "https://api.mythic-beasts.com/beta/dns";

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ApiResponse {
        pub error: Option<String>,
        pub errors: Option<Vec<String>>,
        pub message: Option<String>,
        pub records_added: Option<u32>,
        pub records_removed: Option<u32>,
        pub records: Option<Vec<Record>>,
    }

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

        // #[serde(flatten)]
        // pub extra: std::collections::HashMap<String, serde_json::Value>,

        #[serde(skip)]
        pub _template: Option<bool>,
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

    pub fn build_api_endpoint(app: &ArgMatches) -> String {
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

        endpoint
    }

    pub fn search(url: String, username: &str, password: Option<&str>) -> Result<Option<Vec<Record>>, Box<dyn Error>> {
        let response = reqwest::blocking::Client::new()
            .get(&url)
            .basic_auth(username, password)
            .send()?;

        let text = response.text()?;
        log::trace!("Received response: {}", &text);

        let result: ApiResponse = serde_json::from_str(&text)?;
        log::trace!("{:#?}", result);

        if let Some(e) = result.error {
            return Err(format!("Unable to get search results. Reason: {}", e).into());
        }

        Ok(result.records)
    }

    pub fn delete(app: &ArgMatches, username: &str, password: Option<&str>) -> Result<u32, Box<dyn Error>> {
        let url = build_api_endpoint(app);
        let response = reqwest::blocking::Client::new()
            .delete(&url)
            .basic_auth(username, password)
            .send()?;

        let response_status = response.status();
        let text = response.text()?;
        log::trace!("Received response: {}", &text);

        let result: ApiResponse = serde_json::from_str(&text)?;
        log::trace!("{:#?}", result);

        if response_status.is_client_error() {
            if response_status == reqwest::StatusCode::BAD_REQUEST {
                if let Some(e) = result.errors {
                    return Err(format!("Unable to delete selected record(s). Reasons: \n - {}", e.join("\n - ")).into());
                }
            }

            if let Some(e) = result.error {
                return Err(format!("Unable to delete selected record(s). Reason: {}", e).into());
            }
        }

        if let Some(r) = result.records_removed {
            return Ok(r);
        }

        Ok(0)
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

}


#[allow(dead_code)]
fn get_my_public_ip() -> Result<IpAddr, Box<dyn Error>> {
    let opendns_ip = reqwest::blocking::get("https://diagnostic.opendns.com/myip")?.text()?;
    let ipify_ip = reqwest::blocking::get("https://api6.ipify.org")?.text()?;

    let opendns_ip = IpAddr::from_str(&opendns_ip)?;
    let ipify_ip = IpAddr::from_str(&ipify_ip)?;

    log::debug!("OpenDNS IPAddr: {:?}", opendns_ip);
    log::debug!("IPify IP: {:?}", ipify_ip);

    if opendns_ip != ipify_ip {
        return Err("OpenDNS and IPify services responded with different IPs.".into());
    }

    Ok(opendns_ip)
}
