use aws_sdk_ec2::client::Client as EC2Client;
use aws_sdk_ec2::model::{
    AddPrefixListEntry, ManagedPrefixList, PrefixListState, RemovePrefixListEntry,
};
use color_eyre::{eyre::eyre, Report, Result};
use ipnet::IpNet;
use tokio::time::{interval, timeout, Duration, MissedTickBehavior};

// pub use self::error::AWSError;

// pub type AWSResult<T> = Result<T, AWSError>;

// pub struct Entry {
//     cidr: IpNet,
//     description: String,
// }
//
// impl TryFrom<&PrefixListEntry> for Entry {
//     type Error = Report;
//
//     fn try_from(value: &PrefixListEntry) -> Result<Self> {
//         let cidr = IpNet::from_str(value.cidr.as_ref().ok_or_else(|| eyre!("empty cidr"))?)?;
//         let description = value.description.clone().unwrap_or_else(String::new);
//         Ok(Self { cidr, description })
//     }
// }

pub struct AWSClient {
    ec2_client: EC2Client,
    // prefix_list_v4_id: String,
    // prefix_list_v6_id: String,
    description: String,
}

impl AWSClient {
    pub fn new(ec2_client: EC2Client, description: &str) -> Self {
        Self {
            ec2_client,
            description: description.to_string(),
        }
    }

    pub async fn get_prefix_list(&self, prefix_list_id: &str) -> Result<ManagedPrefixList> {
        let response = self
            .ec2_client
            .describe_managed_prefix_lists()
            .prefix_list_ids(prefix_list_id)
            .send()
            .await?;

        // This should only return 0 or 1 prefix lists, any more is an error
        if response.prefix_lists.is_none() || response.prefix_lists.as_ref().unwrap().is_empty() {
            return Err(eyre!("Prefix list {} was not found.", prefix_list_id));
        }

        let prefix_lists = response.prefix_lists.unwrap();
        if response.next_token.is_some() || prefix_lists.len() > 1 {
            return Err(eyre!(
                "Found too many prefix lists! This shouldn't happen..."
            ));
        }

        Ok(prefix_lists[0].clone())
    }

    // pub async fn get_v4_entries(&self) -> Result<Vec<Entry>> {
    //     self.get_prefix_list_entries(&self.prefix_list_v4_id).await
    // }
    //
    // pub async fn get_v6_entries(&self) -> Result<Vec<Entry>> {
    //     self.get_prefix_list_entries(&self.prefix_list_v6_id).await
    // }
    //
    // async fn get_prefix_list_entries(&self, prefix_list_id: &str) -> Result<Vec<Entry>> {
    //     let mut token = None;
    //     let mut total_entries: Vec<Entry> = Vec::new();
    //
    //     loop {
    //         let response = self
    //             .ec2_client
    //             .get_managed_prefix_list_entries()
    //             .prefix_list_id(prefix_list_id)
    //             .set_next_token(token.clone())
    //             .send()
    //             .await?;
    //
    //         if let Some(entries) = response.entries {
    //             entries.iter().for_each(|entry| match entry.try_into() {
    //                 Ok(ip) => total_entries.push(ip),
    //                 Err(err) => warn!("Failed to parse IP from Managed Prefix List entry: {}", err),
    //             });
    //         };
    //
    //         token = response.next_token;
    //         if token.is_none() {
    //             break;
    //         }
    //     }
    //
    //     Ok(total_entries)
    // }

    /// Modify the prefix list by adding and / or removing an entry.
    pub async fn modify_entries(
        &self,
        prefix_list: &ManagedPrefixList,
        add: Option<&IpNet>,
        remove: Option<&IpNet>,
    ) -> Result<ManagedPrefixList> {
        let add_entries = add.map(|net| {
            vec![AddPrefixListEntry::builder()
                .cidr(net.to_string())
                .description(&self.description)
                .build()]
        });
        let remove_entries = remove.map(|net| {
            vec![RemovePrefixListEntry::builder()
                .cidr(net.to_string())
                .build()]
        });
        let response = self
            .ec2_client
            .modify_managed_prefix_list()
            .prefix_list_id(prefix_list.prefix_list_id.as_ref().unwrap())
            .set_current_version(prefix_list.version)
            .set_add_entries(add_entries)
            .set_remove_entries(remove_entries)
            .send()
            .await?;
        response
            .prefix_list
            .ok_or_else(|| eyre!("Modify Prefix List didn't return a prefix list."))
    }

    pub async fn wait_for_state(
        &self,
        prefix_list_id: &str,
        state: PrefixListState,
        wait_timeout: Option<u64>,
    ) -> Result<ManagedPrefixList> {
        timeout(
            Duration::from_secs(wait_timeout.unwrap_or(60)),
            async move {
                let mut interval_timer = interval(Duration::from_secs(10));
                interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

                loop {
                    interval_timer.tick().await;
                    let mpl = self.get_prefix_list(prefix_list_id).await?;
                    if mpl.state.as_ref() == Some(&state) {
                        return Ok::<ManagedPrefixList, Report>(mpl);
                    }
                }
            },
        )
        .await?
    }
}
