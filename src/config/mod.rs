use clap::{crate_name, crate_version, App, AppSettings, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug)]
pub struct Config {
    // pub instance_id: String,
    pub prefix_list_id: String,
    pub description: String,
    pub external_ip: Option<IpAddr>,
    pub verbose: bool,
    pub cleanup: bool,
    pub interval: u64,
}

impl Config {
    pub fn from_args() -> Self {
        let matches = App::new(crate_name!())
            .version(crate_version!())
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::DeriveDisplayOrder)
            .arg(
                Arg::with_name("cleanup")
                    .long("cleanup")
                    .short("c")
                    .takes_value(false)
                    .required(false)
                    .multiple(false)
                    .help("Only clean up the rules"),
            )
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .takes_value(false)
                    .required(false)
                    .multiple(false)
                    .help("Enable debug logging"),
            )
            .arg(
                Arg::with_name("ip")
                    .long("ip")
                    .takes_value(true)
                    .value_name("EXT IP")
                    .required(false)
                    .multiple(false)
                    .help("External IP")
                    .validator(check_ip),
            )
            .arg(
                Arg::with_name("prefix_list_id")
                    .short("p")
                    .long("prefix-list-id")
                    .value_name("PREFIX LIST ID")
                    .takes_value(true)
                    .required(true)
                    .multiple(false)
                    .help("AWS prefix list ID")
                    .validator(check_prefix_list_format),
            )
            .arg(
                Arg::with_name("description")
                    .short("d")
                    .long("description")
                    .value_name("DESCRIPTION")
                    .takes_value(true)
                    .required(true)
                    .multiple(false)
                    .help("Prefix list entry description")
                    .validator(check_description),
            )
            .arg(
                Arg::with_name("interval")
                    .long("interval")
                    .short("i")
                    .takes_value(true)
                    .value_name("interval")
                    .required(false)
                    .multiple(false)
                    .help("Interval in seconds between external IP checks")
                    .default_value("60")
                    .validator(check_interval),
            )
            .get_matches();

        let interval: u64 = matches.value_of("interval").unwrap().parse().unwrap();
        let prefix_list_id = matches.value_of("prefix_list_id").unwrap().to_string();
        let description = matches.value_of("description").unwrap().to_string();
        let verbose = matches.is_present("verbose");
        let cleanup = matches.is_present("cleanup");

        let external_ip = match matches.value_of("ip") {
            None => None,
            Some(ip_str) => Some(IpAddr::from_str(ip_str).unwrap()),
        };

        Self {
            interval,
            prefix_list_id,
            description,
            external_ip,
            verbose,
            cleanup,
        }
    }
}

fn check_prefix_list_format(pl: String) -> Result<(), String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?i:pl-([[:alnum:]]{8}|[[:alnum:]]{17}))\z").unwrap();
    }
    match RE.is_match(&pl) {
        true => Ok(()),
        false => Err("the expected format is 'pl-1234567890abcdef0'".to_string()),
    }
}

fn check_description(desc: String) -> Result<(), String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?i:[[:alnum:]]{0, 255})\z").unwrap();
    }
    match RE.is_match(&desc) {
        true => Ok(()),
        false => Err("must contain up to 255 alphanumeric characters".to_string()),
    }
}

fn check_ip(value: String) -> Result<(), String> {
    IpAddr::from_str(&value).map_err(|err| err.to_string())?;
    Ok(())
}

fn check_interval(value: String) -> Result<(), String> {
    let int_value = value.parse::<u64>().map_err(|err| err.to_string())?;
    if int_value < 1 {
        return Err("Interval should be at least one second".to_string());
    }
    Ok(())
}
