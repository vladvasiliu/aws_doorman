use std::error::Error;
use std::fmt;

use clap::{crate_name, crate_version, App, AppSettings, Arg};
use std::fmt::Formatter;
use std::net::{IpAddr, AddrParseError};
use std::str::FromStr;

#[derive(Debug)]
pub struct Config {
    pub instance_id: String,
    pub sg_id: String,
    pub sg_desc: String,
    pub external_ip: Option<IpAddr>,
    pub debug: bool,
}

impl Config {
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
            .get_matches();

        let instance_id = matches.value_of("instance_id").unwrap().to_string();
        let sg_id = matches.value_of("sg_id").unwrap().to_string();
        let sg_desc = matches.value_of("sg_desc").unwrap().to_string();
        let debug = matches.is_present("debug");

        let external_ip = match matches.value_of("ip") {
            None => None,
            Some(ip_str) => Some(IpAddr::from_str(ip_str)?),
        };

        Ok(Self {
            instance_id,
            sg_id,
            sg_desc,
            external_ip,
            debug,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MalformedIP(AddrParseError)
}

impl Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedIP(err) => write!(f, "Failed to parse IP address: {}", err)
        }
    }
}

impl From<AddrParseError> for ConfigError {
    fn from(err: AddrParseError) -> Self {
        Self::MalformedIP(err)
    }
}
