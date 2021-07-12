mod aws;
mod config;
mod notification;

use crate::aws::AWSClient;
use crate::config::Config;
use crate::notification::notify;

use aws_sdk_ec2::client::Client;
use aws_sdk_ec2::model::{ManagedPrefixList, PrefixListState};
use color_eyre::{Report, Result};
use ipnet::IpNet;
use log::{debug, error, info, LevelFilter};
use query_external_ip::Consensus;
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
    let ec2_client = Client::from_env();
    let aws_client = AWSClient::new(ec2_client, "test-desc");

    if config.cleanup {
        info!("Running in cleanup mode...");
        aws_client.cleanup(&config.prefix_list_id).await?;
        info!("Done!");
        return Ok(());
    }

    let mut timer = interval(Duration::from_secs(config.interval));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    info!(
        "Sleeping {} seconds between external IP checks.",
        config.interval
    );

    let mut current_cidr: Option<IpNet> = None;
    let mut current_prefix_list: ManagedPrefixList =
        aws_client.get_prefix_list(&config.prefix_list_id).await?;

    loop {
        tokio::select! {
            _ = timer.tick() => {
                match Consensus::get().await.map_err(Report::from) {
                    Err(err) => {
                        error!("Failed to retrieve external IP: {}", err);
                        notify("Failed to retrieve external IP.", "", true)?;
                        continue;
                    }
                    Ok(consensus) => {
                        let new_ip = consensus.v4();
                        if new_ip.is_none() {
                            error!("Failed to retrieve external IP. None found...");
                            notify("Failed to retrieve external IP.", "No IP found...", true)?;
                            continue;
                        }

                        // This works because we know that `new_ip` is a valid IpV4
                        let new_cidr = new_ip.map(|ip| {format!("{}/32", ip).parse::<IpNet>().unwrap()});

                        if new_cidr == current_cidr {
                            debug!("External IP didn't change.");
                            continue;
                        }

                        let add = new_cidr.iter().collect();
                        let remove = current_cidr.iter().collect();
                        match aws_client.modify_entries(&current_prefix_list, add, remove).await {
                            Err(err) => error!("Failed to modify prefix list: {:#?}", err),
                            Ok(mpl) => {
                                let new_prefix_list = aws_client.wait_for_state(&mpl.prefix_list_id.unwrap(), PrefixListState::ModifyComplete, None).await?;
                                info!("Updated prefix list IP to {}", new_cidr.unwrap());
                                notify("Updated prefix list", &format!("New IP: {}", new_cidr.unwrap()), false)?;
                                current_prefix_list = new_prefix_list;
                            }
                        }

                        current_cidr = new_cidr;
                    }
                }
            }
            _ = ctrl_c() => {
                info!("Received ^C. Cleaning up...");
                aws_client.cleanup(&config.prefix_list_id).await?;
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
