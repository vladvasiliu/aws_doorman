use std::{net::IpAddr, result::Result};

use rusoto_ec2::{
    DescribeInstancesRequest, DescribeSecurityGroupsRequest, Ec2, Ec2Client, Instance, Reservation,
};

use crate::aws::error::{AWSClientError, InstanceError, SecurityGroupError};
use crate::aws::helpers::{get_only_item, get_public_ip, has_security_group, is_running};

mod error;
mod helpers;

pub struct AWSClient {
    pub ec2_client: Ec2Client,
    pub instance_id: String,
    pub sg_id: String,
}

impl AWSClient {
    pub fn is_instance_sane(&self, instance: &Instance) -> Result<bool, InstanceError> {
        is_running(instance)?;
        has_security_group(instance, &self.sg_id)?;
        Ok(true)
    }

    async fn get_reservations(&self) -> Result<Option<Vec<Reservation>>, AWSClientError> {
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

    pub async fn get_instance_ip(&self) -> Result<IpAddr, AWSClientError> {
        let reservations = self.get_reservations().await?;
        let ip = self.get_ip_from_reservations(reservations)?;
        println!("IP: {}", ip);
        Ok(ip)
    }

    //
    // pub async fn get_security_group(&self) -> Result<(), AWSClientError> {
    //     let dg_res = self
    //         .ec2_client
    //         .describe_security_groups(DescribeSecurityGroupsRequest {
    //             group_ids: Some(vec![self.sg_id.clone()]),
    //             ..Default::default()
    //         })
    //         .await?;
    //
    //     let sg = get_only_item(&dg_res.security_groups)?;
    //     println!("{:#?}", sg);
    //     Ok(())
    // }
}
