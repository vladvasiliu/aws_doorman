use log::{error, info, warn};
use std::fmt;

use rusoto_ec2::{
    AuthorizeSecurityGroupIngressRequest, DescribeSecurityGroupsRequest, Ec2, Ec2Client,
    IpPermission, IpRange, RevokeSecurityGroupIngressRequest, SecurityGroup,
};

use crate::aws::error::{
    AWSClientError, SGAuthorizeIngressError, SGClientResult, SecurityGroupError,
};
use crate::aws::helpers::{get_only_item, ips_for_rule_in_sg};
use std::fmt::Formatter;

mod error;
pub mod helpers;

#[derive(Clone, Debug, Default)]
pub struct IPRule {
    pub id: String,
    pub ip: String,
    pub from_port: i64,
    pub to_port: i64,
    pub ip_protocol: String,
}

impl From<IPRule> for IpPermission {
    fn from(ip_rule: IPRule) -> Self {
        let ip_range = IpRange {
            description: Some(ip_rule.id),
            cidr_ip: Some(ip_rule.ip),
        };

        Self {
            from_port: Some(ip_rule.from_port),
            to_port: Some(ip_rule.to_port),
            ip_protocol: Some(ip_rule.ip_protocol),
            ip_ranges: Some(vec![ip_range]),
            ..Default::default()
        }
    }
}

impl IPRule {
    pub fn to_ip_permission_with_ips(&self, ips: &[&str]) -> IpPermission {
        let ip_ranges = ips
            .iter()
            .map(|s| IpRange {
                cidr_ip: Some(s.to_string()),
                ..Default::default()
            })
            .collect();
        IpPermission {
            from_port: Some(self.from_port),
            to_port: Some(self.to_port),
            ip_protocol: Some(self.ip_protocol.clone()),
            ip_ranges: Some(ip_ranges),
            ..Default::default()
        }
    }
}

impl fmt::Display for IPRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let range = if self.from_port < self.to_port {
            format!("{} - {}", self.from_port, self.to_port)
        } else {
            format!("{}", self.from_port)
        };
        write!(f, "{} {} from {}", self.ip_protocol, range, self.ip)
    }
}

impl PartialEq<IpPermission> for IPRule {
    fn eq(&self, other: &IpPermission) -> bool {
        Some(self.from_port) == other.from_port
            && Some(self.to_port) == other.to_port
            && Some(&self.ip_protocol.to_lowercase()) == other.ip_protocol.as_ref()
    }
}

impl PartialEq<IPRule> for IpPermission {
    fn eq(&self, other: &IPRule) -> bool {
        self.from_port == Some(other.from_port)
            && self.to_port == Some(other.to_port)
            && self.ip_protocol.as_ref() == Some(&other.ip_protocol.to_lowercase())
    }
}

pub struct AWSClient {
    pub ec2_client: Ec2Client,
    pub sg_id: String,
}

impl AWSClient {
    async fn get_security_groups(&self) -> SGClientResult<Option<Vec<SecurityGroup>>> {
        let dsg_res = self
            .ec2_client
            .describe_security_groups(DescribeSecurityGroupsRequest {
                group_ids: Some(vec![self.sg_id.clone()]),
                ..Default::default()
            })
            .await
            .map_err(|err| {
                error!("Failed to retrieve security group: {}", err);
                SecurityGroupError::from(err)
            })?;

        Ok(dsg_res.security_groups)
    }

    async fn authorize_sg_ingress(&self, rules: Vec<IPRule>) -> SGClientResult<()> {
        let ip_permissions: Vec<IpPermission> =
            rules.iter().map(|rule| rule.clone().into()).collect();
        let request = AuthorizeSecurityGroupIngressRequest {
            ip_permissions: Some(ip_permissions),
            group_id: Some(self.sg_id.to_string()),
            ..Default::default()
        };

        self.ec2_client
            .authorize_security_group_ingress(request)
            .await?;
        Ok(())
    }

    async fn revoke_sg_ingress(&self, ip_permissions: Vec<IpPermission>) -> SGClientResult<()> {
        let request = RevokeSecurityGroupIngressRequest {
            group_id: Some(self.sg_id.to_owned()),
            ip_permissions: Some(ip_permissions),
            ..Default::default()
        };

        self.ec2_client
            .revoke_security_group_ingress(request)
            .await?;
        Ok(())
    }

    /// Removes all IPs with the configured id and given rules
    pub async fn sg_cleanup(&self, rules: &[IPRule]) -> SGClientResult<()> {
        let sec_groups = self.get_security_groups().await?;
        let sg = get_only_item(&sec_groups)?;
        let authorized_ips: Vec<&str> = rules
            .iter()
            .flat_map(|ip_rule| ips_for_rule_in_sg(ip_rule, sg))
            .collect();
        let ip_permissions: Vec<IpPermission> = rules
            .iter()
            .map(|ip_rule| ip_rule.to_ip_permission_with_ips(&authorized_ips))
            .collect();
        if authorized_ips.is_empty() {
            info!("Nothing to delete!")
        } else {
            self.revoke_sg_ingress(ip_permissions).await?;
        };
        Ok(())
    }

    /// Authorize the configured rules
    ///
    /// Will log a warning if a rule (proto / port / ip) is already present
    pub async fn sg_authorize(&self, rules: &[IPRule]) -> SGClientResult<()> {
        // Looping over the rules in order to allow the request to fail in case of duplication
        // Calling the EC2 API with several rules will fail completely if one of them is duplicated.
        for rule in rules {
            match self.authorize_sg_ingress(vec![rule.clone()]).await {
                Ok(()) => (),
                Err(AWSClientError::Service(SecurityGroupError::AuthorizeIngressError(
                    SGAuthorizeIngressError::DuplicateRule(_),
                ))) => {
                    warn!("Duplicate rule: {}", rule);
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
}
