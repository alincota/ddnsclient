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

    let mut provider = providers::init_provider(provider);
    provider.set_credentials(get_provider_credentials(&provider, config));

    let subcommand = match app.subcommand() {
        ("ddns", Some(ddns)) => provider.dynamic_dns(ddns),
        ("update", Some(upd)) => {
            let records: Vec<providers::Record> = match upd.values_of("records") {
                Some(rcds) => process_dns_records(rcds.map(|ln| ln.to_string())),
                None => process_dns_records(io::stdin().lock().lines().map(|ln| ln.unwrap())),
            };

            provider.update(upd, &records)
        },
        ("delete", Some(del)) => provider.delete(del),
        _ => Ok(false),
    };

    match subcommand {
        Ok(subcmd_ran) => {
            if subcmd_ran {
                return ();
            }

            match provider.search(&app) {
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
                },
            }
        },
        Err(e) => {
            log::error!("{}", e);
            process::exit(exitcode::UNAVAILABLE);
        },
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
}

fn process_dns_records<I>(strings: I) -> Vec<providers::Record> where I: IntoIterator<Item = String> + std::fmt::Debug {
    let mut dns_records: Vec<providers::Record> = Vec::new();

    for string in strings {
        let result: Result<Vec<providers::Record>, serde_json::Error> = serde_json::from_str(&string);

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
