use log::{error, info, LevelFilter};
use rusoto_core::Region;
use rusoto_ec2::Ec2Client;

use crate::aws::{AWSClient, AWSError};
use crate::config::Config;
use std::collections::HashSet;

mod aws;
mod config;

#[tokio::main]
async fn main() {
    let config = Config::from_args();

    let log_level = match config.verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };
    setup_logger(log_level).unwrap();

    let ec2_client = Ec2Client::new(Region::EuWest3);
    let aws_client = AWSClient {
        ec2_client: &ec2_client,
        prefix_list_id: &config.prefix_list_id,
        entry_description: &config.description,
    };

    let prefix_list = aws_client.get_prefix_list().await.unwrap();
    let ip_set: HashSet<&str> = [].iter().copied().collect();
    match aws_client.update_ips(prefix_list, ip_set).await {
        Ok(_) => info!("Done cleaning up"),
        Err(AWSError::NothingToDo(err)) => info!("{}", err),
        Err(err) => error!("Something went wrong: {}", err),
    }
}

// async fn work(config: Config) -> Result<(), Box<dyn Error>> {
//     let ip_rules = vec![IPRule {
//         id: config.sg_desc.to_owned(),
//         from_port: config.from_port,
//         to_port: config.to_port,
//         ip_protocol: config.ip_protocol,
//     }];
//
//     let mut timer = interval(Duration::from_secs(config.interval));
//     info!(
//         "Sleeping {} seconds between external IP checks.",
//         config.interval
//     );
//     let mut current_ip: Option<IpAddr> = None;
//     loop {
//         tokio::select! {
//             _ = timer.tick() => {
//                 let new_ip = get_ip().await;
//                 if new_ip.is_none() {
//                     error!("Failed to determine external ip.");
//                     continue;
//                 };
//                 if new_ip == current_ip {
//                     info!("External IP didn't change.");
//                     continue;
//                 }
//                 current_ip = new_ip;
//                 let external_ip = current_ip.unwrap().to_string().add("/32");
//                 info!("Got new external IP: {}", external_ip);
//                 aws_client.sg_cleanup(&ip_rules).await?;
//                 aws_client.sg_authorize(&ip_rules, &[&external_ip]).await?;
//             }
//             _ = ctrl_c() => {
//                 info!("Received ^C. Cleaning up...");
//                 aws_client.sg_cleanup(&ip_rules).await?;
//                 break;
//             }
//         }
//     }
//     Ok(())
// }

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
