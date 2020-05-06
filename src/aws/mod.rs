use log::error;
use std::{net::IpAddr, result::Result};

use rusoto_ec2::{
    AuthorizeSecurityGroupIngressRequest, DescribeInstancesRequest, DescribeSecurityGroupsRequest,
    Ec2, Ec2Client, Instance, IpPermission, IpRange, Reservation, SecurityGroup,
};

use crate::aws::error::{
    AWSClientError, InstanceError, SGAuthorizeIngressError, SGClientResult, SecurityGroupError,
};
use crate::aws::helpers::{get_only_item, get_public_ip, has_security_group, is_running};

mod error;
mod helpers;

#[derive(Debug)]
pub struct IPRule {
    pub id: String,
    pub ip: IpAddr,
    pub from_port: i64,
    pub to_port: i64,
}

pub struct AWSClient {
    pub ec2_client: Ec2Client,
    pub instance_id: String,
    pub sg_id: String,
    pub rule: IPRule,
}

impl AWSClient {
    //
    // Instance related
    //
    pub fn is_instance_sane(&self, instance: &Instance) -> Result<bool, InstanceError> {
        is_running(instance)?;
        has_security_group(instance, &self.sg_id)?;
        Ok(true)
    }

    async fn get_reservations(
        &self,
    ) -> Result<Option<Vec<Reservation>>, AWSClientError<InstanceError>> {
        let di_res = self
            .ec2_client
            .describe_instances(DescribeInstancesRequest {
                instance_ids: Some(vec![self.instance_id.clone()]),
                ..Default::default()
            })
            .await?;
        Ok(di_res.reservations)
    }

    fn get_ip_from_reservations(
        &self,
        reservations: Option<Vec<Reservation>>,
    ) -> Result<IpAddr, InstanceError> {
        // We're expecting one and only one instance,
        // so there should only be one reservation with one instance
        let reservation = get_only_item(&reservations)?;
        let instance = get_only_item(&reservation.instances)?;

        self.is_instance_sane(instance)?;
        let public_ip = get_public_ip(instance)?;
        Ok(public_ip)
    }

    pub async fn get_instance_ip(&self) -> Result<IpAddr, AWSClientError<InstanceError>> {
        let reservations = self.get_reservations().await?;
        let ip = self.get_ip_from_reservations(reservations)?;
        println!("IP: {}", ip);
        Ok(ip)
    }

    //
    // Security Group related
    //

    pub async fn get_security_groups(&self) -> SGClientResult<Option<Vec<SecurityGroup>>> {
        let dsg_res = self
            .ec2_client
            .describe_security_groups(DescribeSecurityGroupsRequest {
                group_ids: Some(vec![self.sg_id.clone()]),
                ..Default::default()
            })
            .await
            .or_else(|err| {
                error!("Failed to retrieve security group: {}", err);
                Err(err)
            })?;

        Ok(dsg_res.security_groups)
    }

    pub async fn is_rule_in_sg(&self) -> SGClientResult<bool> {
        let sg_res = self.get_security_groups().await?;
        let sg = get_only_item(&sg_res).map_err(SecurityGroupError::from)?;

        let ip_permissions = match &sg.ip_permissions {
            Some(perm_vec) => perm_vec,
            _ => return Ok(false),
        };

        for ip_permission in ip_permissions {
            let ip_ranges = match &ip_permission.ip_ranges {
                Some(range_vec) => range_vec,
                _ => continue,
            };

            for range in ip_ranges {
                if range.description.as_ref() == Some(&self.rule.id) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn authorize_sg_ingress(&self) -> SGClientResult<()> {
        let request = AuthorizeSecurityGroupIngressRequest {
            ip_permissions: self.get_ip_permissions(),
            group_id: Some(self.sg_id.to_string()),
            ..Default::default()
        };

        self.ec2_client
            .authorize_security_group_ingress(request)
            .await?;
        Ok(())
    }

    fn get_ip_permissions(&self) -> Option<Vec<IpPermission>> {
        let ip_range = IpRange {
            cidr_ip: Some("10.1.1.1/32".to_string()),
            description: Some("test sg2".to_string()),
        };

        let ip_perm = IpPermission {
            from_port: Some(9000),
            to_port: Some(10000),
            ip_protocol: Some("tcp".to_string()),
            ip_ranges: Some(vec![ip_range]),
            ..Default::default()
        };

        Some(vec![ip_perm])
    }
}
