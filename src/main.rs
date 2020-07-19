mod config;
mod providers;

extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use config::Configuration;
use providers::*;

use std::error::Error; // used for get_my_public_ip()
use std::net::IpAddr; // used for get_my_public_ip()
use std::str::FromStr; // used for get_my_public_ip()
use std::process;
use std::io;
use std::io::prelude::*;
// use std::fs;

use clap::{Arg, App, SubCommand};


fn main() {
    let app = App::new("DDNS Client")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .after_help("SUPPORTED PROVIDERS:\n - Mythic Beasts [mythic-beasts] (https://www.mythic-beasts.com/support/api/dnsv2)")
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
            .help("The record type (e.g A, AAAA, MX, CNAME, TXT)")
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
        .arg(Arg::with_name("config-path")
            .short("c")
            .long("config")
            .required(true)
            .takes_value(true)
            .number_of_values(1)
            .help("Path to the YAML configuration file.")
        )
        .arg(Arg::with_name("provider")
            .long("provider")
            .takes_value(true)
            .number_of_values(1)
            .possible_values(&["mythic-beasts", "noip"])
            .default_value("mythic-beasts")
            .help("Specify DNS provider to use")
        )
        .arg(Arg::with_name("pretty")
            .long("pretty")
            .takes_value(false)
            .help("Human readable output with pretty format")
        )

        .subcommand(SubCommand::with_name("ddns")
            .about("Create or update an A or AAAA record with the specified hostname, with the data set to the IP address of the client using the API.")
            // .arg(Arg::with_name("hostname")
                // .required(true)
                // .takes_value(true)
                // .value_name("HOSTNAME")
                // .help("The fully-qualified host name")
            // )
        )
        .subcommand(SubCommand::with_name("delete")
            .about("Deletes all records selected by the zone|host|type")
        )
        .subcommand(SubCommand::with_name("update")
            .about("Update all records selected by the zone|host|type")
            .arg(Arg::with_name("records")
                .takes_value(true)
                .number_of_values(1)
                .help("Records provided as JSON. If not provided, it will read from stdin")
            )
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

    let config_path = app.value_of("config-path").unwrap();
    let config = Configuration::from_path(config_path);

    // First check the IP of the client
    /* let ip = get_my_public_ip(); */
    // if let Err(e) = ip {
        // log::error!("{}", e);
        // process::exit(exitcode::UNAVAILABLE);
    // }
    /* let ip = ip.unwrap(); */

    let username = app.value_of("username").unwrap();
    let password = app.value_of("password");
    let provider = app.value_of("provider").expect("Unable to establish which provider to use");

    // let provider = Provider::new(provider).from_config(config);
    let mut provider = providers::init_provider(provider);
    provider.set_credentials(get_provider_credentials(&provider, config));
    // println!("{:#?}", provider);

    let subcommand = match app.subcommand() {
        ("ddns", Some(ddns)) => provider.dynamic_dns(ddns),
        // ("update", Some(update)) => Ok(println!("update subcommand")),
        // ("delete", Some(_delete)) => Ok(println!("delete subcommand")),
        _ => Ok(()),
    };

    if subcommand.is_err() {
        subcommand.map_err(|e| log::error!("{}", e));
        process::exit(exitcode::UNAVAILABLE);
    }
    return ();


    match app.subcommand() {
        ("ddns", Some(ddns)) => {
            let host = ddns.value_of("hostname").unwrap();

            if let Err(e) = mythic_beasts::dynamic_dns(host, username, password) {
                log::error!("{}", e);
                process::exit(exitcode::UNAVAILABLE);
            }

            return ();
        },
        ("update", Some(u)) => {
            let records: Vec<mythic_beasts::Record> = match u.values_of("records") {
                Some(rcds) => process_dns_records(rcds.map(|ln| ln.to_string())),
                None => process_dns_records(io::stdin().lock().lines().map(|ln| ln.unwrap())),
            };

            match mythic_beasts::update(&records, &u, username, password) {
                Ok((added, removed)) => {
                    log::info!("Updated record(s)!");
                    log::debug!("Added {} record(s). Removed {} record(s)", added, removed);
                    return ();
                },
                Err(e) => {
                    log::error!("{}", e);
                    process::exit(exitcode::UNAVAILABLE);
                },
            }
        },
        ("delete", Some(_d)) => {
            match mythic_beasts::delete(&app, username, password) {
                Ok(r) => {
                    log::info!("Deleted {} record(s)", r);
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

    match mythic_beasts::search(&app, username, password) {
        Ok(records) => {
            if app.is_present("pretty") {
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
            }

            match serde_json::to_string(&records) {
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

    pub fn build_api_endpoint(app: &ArgMatches, filter: Option<&str>) -> String {
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

    pub fn search(argm: &ArgMatches, username: &str, password: Option<&str>) -> Result<Option<Vec<Record>>, Box<dyn Error>> {
        let url = build_api_endpoint(argm, None);
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
        let url = build_api_endpoint(app, Some("exclude-generated=true&exclude-template=true"));
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

    pub fn update(records: &Vec<Record>, argm: &ArgMatches, username: &str, password: Option<&str>) -> Result<(u32, u32), Box<dyn Error>> {
        let mut recs = std::collections::HashMap::new();
        recs.insert("records", records);

        let url = build_api_endpoint(argm, Some("exclude-generated=true&exclude-template=true"));
        let response = reqwest::blocking::Client::new()
            .put(&url)
            .basic_auth(username, password)
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
                    return Err(format!("Unable to update selected record(s). Reasons: \n - {}", e.join("\n - ")).into());
                }
            }

            if let Some(e) = result.error {
                return Err(format!("Unable to update selected record(s). Reason: {}", e).into());
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

        Ok((records_added, records_removed))
    }
}

fn process_dns_records<I>(strings: I) -> Vec<mythic_beasts::Record> where I: IntoIterator<Item = String> + std::fmt::Debug {
    let mut dns_records: Vec<mythic_beasts::Record> = Vec::new();

    for string in strings {
        let result: Result<Vec<mythic_beasts::Record>, serde_json::Error> = serde_json::from_str(&string);

        if let Ok(r) = result {
            dns_records.extend(r);
        }
    }

    dns_records
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
