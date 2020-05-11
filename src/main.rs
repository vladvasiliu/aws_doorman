use std::process::exit;

use log::{debug, error, info, LevelFilter};
use rusoto_core::Region;
use rusoto_ec2::Ec2Client;

use crate::aws::{AWSClient, IPRule, helpers::get_only_item};
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

use external_ip::get_ip;

mod aws;
mod config;

#[tokio::main]
async fn main() {
    let config = match Config::from_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration:\n{}", err);
            exit(1)
        }
    };

    let log_level = match config.debug {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };
    setup_logger(log_level).unwrap();

    let my_external_ip = match config.external_ip {
        None => {
            info!("No external IP given, attempting to determine it automatically...");
            get_ip().await.unwrap_or_else(||{
                error!("Failed to determine external ip.");
                exit(1)
            })
        },
        Some(ip) => ip,
    };
    info!("Using external IP {}", my_external_ip);

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
    let ip_rules = vec![
        IPRule {
            id: String::from("test sg rule id"),
            ip: "192.168.1.1/32".to_string(),
            from_port: 9999,
            to_port: 10000,
            ip_protocol: "tcp".to_string(),
        },
        IPRule {
            id: String::from("test sg rule id"),
            ip: "192.168.1.2/32".to_string(),
            from_port: 9999,
            to_port: 10000,
            ip_protocol: "tcp".to_string(),
        }
    ];
    let ec2_client = Ec2Client::new(Region::EuWest3);
    let aws_client = AWSClient {
        ec2_client,
        instance_id: config.instance_id,
        sg_id: config.sg_id,
    };
    // let _instance_ip = aws_client.get_instance_ip().await.or_else(|err| {
    //     error!("Failed to retrieve instance IP: {}", err);
    //     Err(err)
    // })?;
    aws_client.sg_authorize(ip_rules).await?;
    // aws_client.sg_cleanup(vec![ip_rule]).await?;
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
