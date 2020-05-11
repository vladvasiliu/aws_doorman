use std::error::Error;
use std::fmt;

use clap::{crate_name, crate_version, App, AppSettings, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::Formatter;
use std::net::{IpAddr, AddrParseError};
use std::str::FromStr;
use crate::config::error::ConfigError;

pub mod error;


#[derive(Debug)]
pub struct Config {
    pub instance_id: String,
    pub sg_id: String,
    pub sg_desc: String,
    pub external_ip: Option<IpAddr>,
    pub ip_protocol: String,
    pub from_port: i64,
    pub to_port: i64,
    pub debug: bool,
}

impl Config {
    fn is_sane(&self) -> Result<(), ConfigError> {
        if self.from_port < 0 {
            return Err(ConfigError::IncorrectPortRange("Port numbers should be positive".to_string()))
        }
        if self.to_port < self.from_port {
            return Err(ConfigError::IncorrectPortRange("Ports should be in ascending order".to_string()))
        }
        check_sg_format(&self.sg_id)?;
        check_instance_format(&self.instance_id)?;
        check_ip_protocol(&self.ip_protocol)?;
        Ok(())
    }


    pub fn from_args() -> Result<Self, ConfigError> {
        let matches = App::new(crate_name!())
            .version(crate_version!())
            .setting(AppSettings::ColoredHelp)
            .arg(
                Arg::with_name("instance_id")
                    .short("i")
                    .long("instance")
                    .value_name("INSTANCE ID")
                    .takes_value(true)
                    .required(true)
                    .multiple(false)
                    .help("AWS Instance ID"),
            )
            .arg(
                Arg::with_name("sg_id")
                    .short("s")
                    .long("sg-id")
                    .value_name("SECGROUP ID")
                    .takes_value(true)
                    .required(true)
                    .multiple(false)
                    .help("AWS Security Group ID"),
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
                Arg::with_name("ip")
                    .long("ip")
                    .takes_value(true)
                    .value_name("EXT IP")
                    .required(false)
                    .multiple(false)
                    .help("External IP"),
            )
            .arg(
                Arg::with_name("ip_protocol")
                    .long("proto")
                    .takes_value(true)
                    .value_name("IP protocol")
                    .required(true)
                    .multiple(false)
                    .help("IP protocol"),
            )
            .arg(
                Arg::with_name("from_port")
                    .long("from")
                    .takes_value(true)
                    .value_name("from_port")
                    .required(true)
                    .multiple(false)
                    .help("from port"),
            )
            .arg(
                Arg::with_name("to_port")
                    .long("to")
                    .takes_value(true)
                    .value_name("to_port")
                    .required(false)
                    .multiple(false)
                    .help("to port"),
            )
            .get_matches();

        let instance_id = matches.value_of("instance_id").unwrap().to_string();
        let sg_id = matches.value_of("sg_id").unwrap().to_string();
        let sg_desc = matches.value_of("sg_desc").unwrap().to_string();
        let debug = matches.is_present("debug");

        let ip_protocol = matches.value_of("ip_protocol").unwrap().to_string();
        let from_port: i64 = matches.value_of("from_port").unwrap().parse().map_err(ConfigError::MalformedPort)?;

        let to_port: i64 = match matches.value_of("to_port") {
            None => from_port,
            Some(to_port) => to_port.parse().map_err(ConfigError::MalformedPort)?,
        };

        let external_ip = match matches.value_of("ip") {
            None => None,
            Some(ip_str) => Some(IpAddr::from_str(ip_str)?),
        };

        let config = Self {
            instance_id,
            sg_id,
            sg_desc,
            external_ip,
            ip_protocol,
            from_port,
            to_port,
            debug,
        };

        config.is_sane()?;
        Ok(config)
    }
}

fn check_sg_format(sg: &str) -> Result<(), ConfigError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?i:sg-([[:alnum:]]{8}|[[:alnum:]]{17}))\z").unwrap();
    }
    match RE.is_match(sg) {
        true => Ok(()),
        false => Err(ConfigError::MalformedSG)
    }
}

fn check_instance_format(sg: &str) -> Result<(), ConfigError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?i:i-([[:alnum:]]{8}|[[:alnum:]]{17}))\z").unwrap();
    }
    match RE.is_match(sg) {
        true => Ok(()),
        false => Err(ConfigError::MalformedInstance)
    }
}

fn check_ip_protocol(sg: &str) -> Result<(), ConfigError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?i:(tcp)|(udp))\z").unwrap();
    }
    match RE.is_match(sg) {
        true => Ok(()),
        false => Err(ConfigError::MalformedProtocol)
    }
}
