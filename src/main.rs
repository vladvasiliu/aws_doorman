use std::process::exit;

use log::{debug, error, info, LevelFilter};
use rusoto_core::Region;
use rusoto_ec2::Ec2Client;
use tokio::time::{interval, Duration};

use crate::aws::{AWSClient, IPRule};
use crate::config::Config;
use std::error::Error;

use external_ip::get_ip;
use std::net::IpAddr;
use std::ops::Add;

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

    match work(config).await {
        Ok(()) => info!("Done!"),
        Err(err) => {
            debug!("{:#?}", err);
            error!("{}", err);
            exit(1)
        }
    }
}

async fn work(config: Config) -> Result<(), Box<dyn Error>> {
    let ip_rules = vec![IPRule {
        id: config.sg_desc.to_owned(),
        // ip: config.external_ip.unwrap().to_string().add("/32"),
        from_port: config.from_port,
        to_port: config.to_port,
        ip_protocol: config.ip_protocol,
    }];
    let ec2_client = Ec2Client::new(Region::EuWest3);
    let aws_client = AWSClient {
        ec2_client,
        sg_id: config.sg_id.to_owned(),
    };

    let mut timer = interval(Duration::from_secs(120));
    let mut current_ip: Option<IpAddr> = None;
    loop {
        timer.tick().await;
        let new_ip = get_ip().await;
        if new_ip.is_none() {
            error!("Failed to determine external ip.");
            continue;
        };
        if new_ip == current_ip {
            info!("External IP didn't change.");
            continue;
        }
        current_ip = new_ip;
        let external_ip = current_ip.unwrap().to_string().add("/32");
        info!("Got new external IP: {}", external_ip);
        if config.cleanup {
            info!("Cleaning up...");
            aws_client.sg_cleanup(&ip_rules).await?;
        }
        aws_client.sg_authorize(&ip_rules, &[&external_ip]).await?;
    }
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
