use std::process::exit;

use log::{debug, error, info, LevelFilter};
use rusoto_core::Region;
use rusoto_ec2::Ec2Client;

use crate::aws::{AWSClient, IPRule, helpers::get_only_item};
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

mod aws;
mod config;
mod ip;

#[tokio::main]
async fn main() {
    setup_logger(LevelFilter::Debug).unwrap();
    // let my_external_ip = ip::guess().await.unwrap_or_else(|_| exit(1));
    let my_external_ip = IpAddr::from([192, 168, 1, 1]);
    let config = Config::from_args().unwrap();

    match work(config, my_external_ip).await {
        Ok(()) => info!("Done!"),
        Err(err) => {
            debug!("{:#?}", err);
            error!("{}", err);
            exit(1)
        }
    }
}

async fn work(config: Config, external_ip: IpAddr) -> Result<(), Box<dyn Error>> {
    let ip_rule = IPRule {
        id: String::from("test sg rule id"),
        ip: external_ip,
        from_port: 9999,
        to_port: 10000,
        ip_protocol: "tcp".to_string(),
    };
    let ec2_client = Ec2Client::new(Region::EuWest3);
    let aws_client = AWSClient {
        ec2_client,
        instance_id: config.instance_id,
        sg_id: config.sg_id,
        rule: ip_rule,
    };
    // let _instance_ip = aws_client.get_instance_ip().await.or_else(|err| {
    //     error!("Failed to retrieve instance IP: {}", err);
    //     Err(err)
    // })?;
    // aws_client.add_ip_to_security_group().await?;
    // println!("{:#?}", res);

    Ok(())
}

fn setup_logger(level: log::LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                // "[ {} ][ {:5} ][ {:15} ] {}",
                // chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                "[ {:5} ][ {:15} ] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        //        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
