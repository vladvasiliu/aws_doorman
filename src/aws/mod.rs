use std::{net::IpAddr, result::Result};

use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client, Instance, Reservation};

use crate::aws::helpers::get_public_ip;
use crate::aws::{
    error::EC2InstanceError,
    helpers::{has_security_group, is_running},
};

pub mod error;
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

        // We're expecting one and only one instance, so there should only be one reservation
        let reservation: &Reservation = match &di_res.reservations {
            None => return Err(EC2InstanceError::DescribeInstancesReturnedNone),
            Some(x) if x.is_empty() => return Err(EC2InstanceError::DescribeInstancesReturnedNone),
            Some(x) if x.len() > 1 => {
                return Err(EC2InstanceError::DescribeInstancesReturnedTooMany)
            }
            Some(x) => &x[0],
        };

        let instance = match &reservation.instances {
            None => return Err(EC2InstanceError::DescribeInstancesReturnedNone),
            Some(instance_vec) if instance_vec.is_empty() => {
                return Err(EC2InstanceError::DescribeInstancesReturnedNone)
            }
            Some(instance_vec) if instance_vec.len() > 1 => {
                return Err(EC2InstanceError::DescribeInstancesReturnedTooMany)
            }
            Some(instance_vec) => &instance_vec[0],
        };

        self.is_instance_sane(instance)?;
        get_public_ip(instance)
    }
}
