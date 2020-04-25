use std::{net::IpAddr, result::Result};

use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client, Instance, Reservation};

use crate::{aws::error::EC2InstanceError, aws::helpers::is_running};

pub mod error;
mod helpers;

#[derive(Debug)]
pub struct EC2Instance {
    pub id: String,
    pub name: Option<String>,
    pub ip_addresses: Vec<IpAddr>,
}

impl EC2Instance {
    fn from_reservation(reservation: &Reservation) -> Result<Self, EC2InstanceError> {
        let instance: &Instance = match &reservation.instances {
            None => return Err(EC2InstanceError::DescribeInstancesReturnedNone),
            Some(x) if x.is_empty() => return Err(EC2InstanceError::DescribeInstancesReturnedNone),
            Some(x) if x.len() > 1 => {
                return Err(EC2InstanceError::DescribeInstancesReturnedTooMany)
            }
            Some(x) => &x[0],
        };

        // Every instance has an ID
        let id = instance.instance_id.as_deref().unwrap().to_string();

        is_running(&instance.state)?;

        let public_ip: IpAddr = match &instance.public_ip_address {
            None => return Err(EC2InstanceError::InstanceHasNoPublicIP),
            Some(ip) => ip.parse()?,
        };

        Ok(Self {
            id,
            name: None,
            ip_addresses: vec![public_ip],
        })
    }

    pub async fn from_query(id: String, ec2_client: Ec2Client) -> Result<Self, EC2InstanceError> {
        let describe_instance_request = DescribeInstancesRequest {
            instance_ids: Some(vec![id]),
            ..Default::default()
        };
        let describe_instance_result = ec2_client
            .describe_instances(describe_instance_request)
            .await?;

        // We're expecting one and only one instance
        match describe_instance_result.reservations {
            None => Err(EC2InstanceError::DescribeInstancesReturnedNone),
            Some(x) if x.is_empty() => Err(EC2InstanceError::DescribeInstancesReturnedNone),
            Some(x) if x.len() > 1 => Err(EC2InstanceError::DescribeInstancesReturnedTooMany),
            Some(x) => Self::from_reservation(&x[0]),
        }
    }
}
