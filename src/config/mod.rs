use std::error::Error;
use std::fmt;

use blob_uuid::random_blob;
use clap::{crate_name, crate_version, App, AppSettings, Arg};
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Config {
    pub instance_id: String,
    pub sg_id: String,
    pub debug: bool,
    pub uuid: String,
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
                    .long("secgroup")
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
                Arg::with_name("uuid")
                    .short("u")
                    .long("uuid")
                    .takes_value(true)
                    .value_name("UUID")
                    .required(false)
                    .multiple(false)
                    .help("UUID to use"),
            )
            .get_matches();

        let instance_id = matches.value_of("instance_id").unwrap().to_string();
        let sg_id = matches.value_of("sg_id").unwrap().to_string();
        let debug = matches.is_present("debug");
        let uuid = matches
            .value_of("uuid")
            .map_or_else(random_blob, String::from);

        Ok(Self {
            instance_id,
            sg_id,
            debug,
            uuid,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {}

impl Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
