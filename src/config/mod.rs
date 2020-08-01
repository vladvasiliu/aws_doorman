
use clap::{crate_name, crate_version, App, AppSettings, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use std::net::IpAddr;
use std::str::FromStr;
use crate::config::error::ConfigError;
use std::num::ParseIntError;

pub mod error;


#[derive(Debug)]
pub struct Config {
    // pub instance_id: String,
    pub sg_id: String,
    pub sg_desc: String,
    pub external_ip: Option<IpAddr>,
    pub ip_protocol: String,
    pub from_port: i64,
    pub to_port: i64,
    pub debug: bool,
    pub cleanup: bool,
}

impl Config {
    pub fn from_args() -> Result<Self, ConfigError> {
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
                    .help("Only clean up the rules")
            )
            .arg(
                Arg::with_name("debug")
                    .short("d")
                    .long("debug")
                    .takes_value(false)
                    .required(false)
                    .multiple(false)
                    .help("Enable debug logging"),
            )
            // .arg(
            //     Arg::with_name("instance_id")
            //         .short("i")
            //         .long("instance")
            //         .value_name("INSTANCE ID")
            //         .takes_value(true)
            //         .required(true)
            //         .multiple(false)
            //         .help("AWS Instance ID")
            // .validator(check_instance_format)
            // )
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
                Arg::with_name("sg_id")
                    .short("s")
                    .long("sg-id")
                    .value_name("SECGROUP ID")
                    .takes_value(true)
                    .required(true)
                    .multiple(false)
                    .help("AWS Security Group ID")
                    .validator(check_sg_format)
            )
            .arg(
                Arg::with_name("sg_desc")
                    .long("sg-desc")
                    .takes_value(true)
                    .value_name("SG DESC")
                    .required(true)
                    .multiple(false)
                    .help("SG description"),
            )
            .arg(
                Arg::with_name("ip_protocol")
                    .long("proto")
                    .takes_value(true)
                    .value_name("IP protocol")
                    .required(true)
                    .multiple(false)
                    .help("IP protocol")
                    .validator(check_ip_protocol)
            )
            .arg(
                Arg::with_name("from_port")
                    .long("from")
                    .visible_alias("port")
                    .takes_value(true)
                    .value_name("FROM PORT")
                    .required(true)
                    .multiple(false)
                    .help("from port")
                    .validator(check_port_number)
            )
            .arg(
                Arg::with_name("to_port")
                    .long("to")
                    .takes_value(true)
                    .value_name("TO PORT")
                    .required(false)
                    .multiple(false)
                    .help("to port")
                    .validator(check_port_number)
            )
            .get_matches();

        // let instance_id = matches.value_of("instance_id").unwrap().to_string();
        let sg_id = matches.value_of("sg_id").unwrap().to_string();
        let sg_desc = matches.value_of("sg_desc").unwrap().to_string();
        let debug = matches.is_present("debug");
        let cleanup = matches.is_present("cleanup");

        let ip_protocol = matches.value_of("ip_protocol").unwrap().to_string();
        let from_port: i64 = matches.value_of("from_port").unwrap().parse().unwrap();

        let to_port: i64 = match matches.value_of("to_port") {
            None => from_port,
            Some(to_port) => to_port.parse().unwrap(),
        };

        let external_ip = match matches.value_of("ip") {
            None => None,
            Some(ip_str) => Some(IpAddr::from_str(ip_str).unwrap()),
        };

        if to_port < from_port {
            return Err(ConfigError::IncorrectPortRange("Ports should be in ascending order".to_string()))
        }

        Ok(Self {
            // instance_id,
            sg_id,
            sg_desc,
            external_ip,
            ip_protocol,
            from_port,
            to_port,
            debug,
            cleanup,
        })
    }
}

fn check_sg_format(sg: String) -> Result<(), String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?i:sg-([[:alnum:]]{8}|[[:alnum:]]{17}))\z").unwrap();
    }
    match RE.is_match(&sg) {
        true => Ok(()),
        false => Err("expected format is 'sg-1234567890abcdef0'".to_string())
    }
}

// fn check_instance_format(sg: String) -> Result<(), String> {
//     lazy_static! {
//         static ref RE: Regex = Regex::new(r"\A(?i:i-([[:alnum:]]{8}|[[:alnum:]]{17}))\z").unwrap();
//     }
//     match RE.is_match(&sg) {
//         true => Ok(()),
//         false => Err("expected format is 'i-1234567890abcdef0'".to_string())
//     }
// }

fn check_ip_protocol(sg: String) -> Result<(), String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?i:(tcp)|(udp))\z").unwrap();
    }
    match RE.is_match(&sg) {
        true => Ok(()),
        false => Err("expected 'tcp' or 'udp'".to_string())
    }
}

fn check_port_number(value: String) -> Result<(), String> {
    let int_value: i64 = value.parse().or_else(|err: ParseIntError| Err(err.to_string()))?;
    if int_value < 0 || int_value > 65535 {
        return Err("port number should be between 0 and 65535".to_string())
    }
    Ok(())
}

fn check_ip(value: String) -> Result<(), String> {
    IpAddr::from_str(&value).or_else(|err| Err(err.to_string()))?;
    Ok(())
}