use clap::{crate_name, crate_version, App, AppSettings, Arg};

#[derive(Debug)]
pub struct Config {
    pub instance_id: String,
    pub sg_id: String,
    pub debug: bool,
}

impl Config {
    pub fn from_args() -> Self {
        let matches = App::new(crate_name!())
            .version(crate_version!())
            .setting(AppSettings::ColoredHelp)
            .arg(
                Arg::with_name("instance_id")
                    .short('i')
                    .long("instance")
                    .value_name("INSTANCE ID")
                    .takes_value(true)
                    .required(true)
                    .help("AWS Instance ID"),
            )
            .arg(
                Arg::with_name("sg_id")
                    .short('s')
                    .long("secgroup")
                    .value_name("SECGROUP ID")
                    .takes_value(true)
                    .required(true)
                    .help("AWS Security Group ID"),
            )
            .arg(
                Arg::with_name("debug")
                    .short('d')
                    .long("debug")
                    .takes_value(false)
                    .required(false)
                    .multiple(false)
                    .help("Enable debug logging"),
            )
            .get_matches();

        let instance_id = matches.value_of_t_or_exit("instance_id");
        let sg_id = matches.value_of_t_or_exit("sg_id");

        Self {
            instance_id,
            sg_id,
            debug: true,
        }
    }
}
