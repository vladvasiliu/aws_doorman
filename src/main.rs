mod aws;
mod config;

use crate::aws::{AWSClient, AWSError};
use crate::config::Config;

use color_eyre::Result;
use log::{error, info, LevelFilter};
use query_external_ip::Consensus;
use rusoto_core::Region;
use rusoto_ec2::Ec2Client;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use tokio::signal::ctrl_c;
use tokio::time::{interval, Duration, MissedTickBehavior};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = Config::from_args();

    let log_level = match config.verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };
    setup_logger(log_level).unwrap();

    work(config).await?;
    Ok(())
}

async fn work(config: Config) -> Result<()> {
    let ec2_client = Ec2Client::new(Region::default());
    let aws_client = AWSClient {
        ec2_client: &ec2_client,
        prefix_list_id: &config.prefix_list_id,
        entry_description: &config.description,
    };
    let mut prefix_list = aws_client.get_prefix_list().await?;
    let mut timer = interval(Duration::from_secs(config.interval));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    info!(
        "Sleeping {} seconds between external IP checks.",
        config.interval
    );
    let mut current_ip: Option<Ipv4Addr> = None;
    loop {
        tokio::select! {
            _ = timer.tick() => {
                let new_ip = match Consensus::get().await {
                    Ok(c) => c.v4(),
                    Err(err) => {
                        error!("Failed to determine external IP: {}", err);
                        None
                    }
                };
                if new_ip == current_ip {
                    info!("External IP didn't change.");
                    continue;
                }
                current_ip = new_ip;
                let external_ip = format!("{}/32", current_ip.unwrap());
                info!("Got new external IP: {}", external_ip);
                let mut ip_set: HashSet<&str> = HashSet::new();
                ip_set.insert(&external_ip);
                match aws_client.update_ips(&prefix_list, ip_set).await {
                    Err(AWSError::NothingToDo(_)) => (),
                    Err(e) => return Err(e.into()),
                    Ok(_) => prefix_list = aws_client.get_prefix_list().await?,
                }
            }
            _ = ctrl_c() => {
                info!("Received ^C. Cleaning up...");
                let empty_ips = HashSet::new();
                let prefix_list = aws_client.get_prefix_list().await?;
                aws_client.update_ips(&prefix_list, empty_ips).await?;
                break;
            }
        }
    }
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
