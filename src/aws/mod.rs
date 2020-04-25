use std::{net::IpAddr, result::Result};

use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client, Instance};

use crate::aws::error::EC2InstanceError;
use crate::aws::helpers::{get_only_item, get_public_ip, has_security_group, is_running};

mod error;
mod helpers;

pub struct AWSClient {
    pub ec2_client: Ec2Client,
    pub instance_id: String,
    pub sg_id: String,
}

impl AWSClient {
    pub fn is_instance_sane(&self, instance: &Instance) -> Result<bool, EC2InstanceError> {
        is_running(instance)?;
        has_security_group(instance, &self.sg_id)?;
        Ok(true)
    }

    pub async fn get_instance_ip(&self) -> Result<IpAddr, EC2InstanceError> {
        let di_res = self
            .ec2_client
            .describe_instances(DescribeInstancesRequest {
                instance_ids: Some(vec![self.instance_id.clone()]),
                ..Default::default()
            })
            .await?;

        // We're expecting one and only one instance,
        // so there should only be one reservation with one instance
        let reservation = get_only_item(&di_res.reservations)?;
        let instance = get_only_item(&reservation.instances)?;

        self.is_instance_sane(instance)?;
        get_public_ip(instance)
    }
}
