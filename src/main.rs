mod config;
mod providers;

extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use config::{Configuration, Credential};
use providers::*;

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
        .arg(Arg::with_name("username")
            .short("u")
            .long("username")
            .required(true)
            .required_unless("config-path")
            .env("DNSAPICLIENT_USER")
            .takes_value(true)
            .number_of_values(1)
            .help("Authentication username")
        )
        .arg(Arg::with_name("password")
            .short("p")
            .long("password")
            .required(true)
            .required_unless("config-path")
            .takes_value(true)
            .number_of_values(1)
            .env("DNSAPICLIENT_PASS")
            .help("Authentication password")
        )
        .arg(Arg::with_name("config-path")
            .short("c")
            .long("config")
            .required(true)
            .required_unless("username")
            .conflicts_with_all(&["username", "password"])
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
            // TODO: support this style as well - more user friendly
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

    let provider = app.value_of("provider").expect("Unable to establish which provider to use");

    let mut config = Configuration::new();
    if let Some(config_path) = app.value_of("config-path") {
        config = Configuration::from_path(config_path);
    }
    if app.is_present("username") && app.is_present("password") {
        config = Configuration {
            credentials: vec![Credential {
                provider: provider.to_string(),
                user: app.value_of("username").unwrap().to_string(),
                pass: app.value_of("password").unwrap().to_string(),
                zone: None,
                host: None,
                r#type: None,
            }],
        };
    }

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


/// Process DNS records utility function
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
