mod error;

use self::error::AWSError;
use core::fmt;
use rusoto_ec2::{
    AddPrefixListEntry, DescribeManagedPrefixListsRequest, Ec2, Ec2Client,
    GetManagedPrefixListEntriesRequest, ManagedPrefixList, ModifyManagedPrefixListRequest,
    PrefixListEntry, RemovePrefixListEntry,
};
use std::fmt::Formatter;

pub type AWSResult<T> = Result<T, AWSError>;

pub struct AWSClient<'a> {
    pub ec2_client: &'a Ec2Client,
    pub prefix_list_id: &'a str,
    pub entry_description: &'a str,
}

impl<'a> AWSClient<'a> {
    async fn get_managed_prefix_list(&self) -> AWSResult<ManagedPrefixList> {
        let request = DescribeManagedPrefixListsRequest {
            prefix_list_ids: Some(vec![self.prefix_list_id.to_owned()]),
            ..Default::default()
        };
        let result = self
            .ec2_client
            .describe_managed_prefix_lists(request)
            .await?;

        // There should be at most one result, so next_token must be None and prefix_lists must have
        // at most one element.
        if result.next_token.is_some() {
            return Err(AWSError::CardinalityError(
                "Got too many prefix lists from AWS.".to_string(),
            ));
        }
        match result.prefix_lists {
            None => Err(AWSError::CardinalityError(format!(
                "Prefix list `{}` not found.",
                self.prefix_list_id
            ))),
            Some(mpl_vec) => match mpl_vec.len() {
                0 => Err(AWSError::CardinalityError(format!(
                    "Prefix list `{}` not found.",
                    self.prefix_list_id
                ))),
                1 => Ok(mpl_vec[0].clone()),
                _ => Err(AWSError::CardinalityError(
                    "Got too many prefix lists from AWS.".to_string(),
                )),
            },
        }
    }

    /// Returns the prefix list entries for a given version
    async fn get_managed_prefix_entries(
        &self,
        version: Option<i64>,
    ) -> AWSResult<Vec<PrefixListEntry>> {
        let mut entries = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let request = GetManagedPrefixListEntriesRequest {
                next_token,
                prefix_list_id: self.prefix_list_id.to_owned(),
                target_version: version,
                ..Default::default()
            };
            let result = self
                .ec2_client
                .get_managed_prefix_list_entries(request)
                .await?;

            // If there are no entries the result contains Some([]), so unwrap() should be safe.
            entries.append(&mut result.entries.unwrap());

            if result.next_token.is_none() {
                break;
            }
            next_token = result.next_token;
        }

        Ok(entries)
    }

    async fn modify_managed_prefix_list(
        &self,
        prefix_list: &PrefixList,
        add_ips: Option<Vec<&str>>,
        remove_ips: Option<Vec<&str>>,
    ) -> AWSResult<ManagedPrefixList> {
        let request = ModifyManagedPrefixListRequest {
            prefix_list_id: self.prefix_list_id.to_owned(),
            current_version: prefix_list.managed_prefix_list.version,
            add_entries: add_ips.map(|ip_vec| {
                ip_vec
                    .into_iter()
                    .map(|cidr| AddPrefixListEntry {
                        cidr: cidr.to_owned(),
                        description: Some(self.entry_description.to_owned()),
                    })
                    .collect()
            }),
            remove_entries: remove_ips.map(|ip_vec| {
                ip_vec
                    .into_iter()
                    .map(|cidr| RemovePrefixListEntry {
                        cidr: cidr.to_owned(),
                    })
                    .collect()
            }),
            ..Default::default()
        };
        let result = self
            .ec2_client
            .modify_managed_prefix_list(request)
            .await?
            .prefix_list
            .unwrap();
        Ok(result)
    }

    pub async fn get_prefix_list(&self) -> AWSResult<PrefixList> {
        let managed_prefix_list = self.get_managed_prefix_list().await?;
        let managed_prefix_entries = self
            .get_managed_prefix_entries(managed_prefix_list.version)
            .await?;
        Ok(PrefixList {
            managed_prefix_list,
            managed_prefix_entries,
        })
    }

    pub fn get_managed_ips(&self, pl: &'a PrefixList) -> Vec<&'a str> {
        pl.managed_prefix_entries
            .iter()
            .filter_map(|entry| {
                entry.description.as_ref().and_then(|desc| {
                    if desc.eq_ignore_ascii_case(self.entry_description) {
                        entry.cidr.as_deref()
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub async fn cleanup(&self, pl: &PrefixList) -> AWSResult<ManagedPrefixList> {
        let managed_ips = self.get_managed_ips(pl);
        let result = self
            .modify_managed_prefix_list(pl, None, Some(managed_ips))
            .await?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct PrefixList {
    pub managed_prefix_list: ManagedPrefixList,
    pub managed_prefix_entries: Vec<PrefixListEntry>,
}

impl fmt::Display for PrefixList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // All the printed fields are required at Managed Prefix List creation so unwrap() should
        // be safe.
        // https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateManagedPrefixList.html
        let pl_id = self.managed_prefix_list.prefix_list_id.as_ref().unwrap();
        let pl_name = format!(
            " ({})",
            self.managed_prefix_list.prefix_list_name.as_ref().unwrap()
        );
        let pl_version = self.managed_prefix_list.version.unwrap();
        let addr_family = self.managed_prefix_list.address_family.as_ref().unwrap();
        let max_entries = self.managed_prefix_list.max_entries.unwrap();
        write!(
            f,
            "ID: {}{} version {}; family: {}; max entries: {}",
            pl_id, pl_name, pl_version, addr_family, max_entries
        )
    }
}
