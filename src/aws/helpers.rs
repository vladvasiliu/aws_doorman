use rusoto_ec2::{Instance, InstanceState};

use crate::aws::error::AWSClientError;
use std::net::IpAddr;

pub fn is_running(instance: &Instance) -> Result<bool, AWSClientError> {
    match &instance.state {
        Some(InstanceState { code: Some(16), .. }) => Ok(true),
        Some(state) => {
            let code = state.code.unwrap_or_default();
            let name = state.name.as_deref().unwrap_or_default();
            Err(AWSClientError::InstanceHasIncorrectState(format!(
                "{} - {}",
                code, name
            )))
        }
        None => Err(AWSClientError::InstanceHasIncorrectState(
            "unknown".to_string(),
        )),
    }
}

pub fn has_security_group(instance: &Instance, sg_id: &str) -> Result<bool, AWSClientError> {
    if let Some(sg_vec) = &instance.security_groups {
        if sg_vec
            .iter()
            .any(|x| x.group_id == Some(String::from(sg_id)))
        {
            return Ok(true);
        }
    }

    Err(AWSClientError::SecurityGroupNotAttached)
}

pub fn get_public_ip(instance: &Instance) -> Result<IpAddr, AWSClientError> {
    match &instance.public_ip_address {
        None => Err(AWSClientError::InstanceHasNoPublicIP),
        Some(ip) => ip
            .parse()
            .map_err(AWSClientError::InstanceHasMalformedPublicIP),
    }
}

pub fn get_only_item<T>(item_vec: &Option<Vec<T>>) -> Result<&T, AWSClientError> {
    match item_vec {
        Some(item_vec) if item_vec.len() == 1 => Ok(&item_vec[0]),
        Some(item_vec) if item_vec.len() > 1 => {
            Err(AWSClientError::DescribeInstancesReturnedTooMany)
        }
        _ => Err(AWSClientError::DescribeInstancesReturnedNone),
    }
}
