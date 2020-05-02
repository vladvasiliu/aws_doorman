use rusoto_ec2::{Instance, InstanceState};

use crate::aws::error::{AWSClientError, CardinalityError, InstanceError};
use std::net::IpAddr;

pub fn is_running(instance: &Instance) -> Result<bool, InstanceError> {
    match &instance.state {
        Some(InstanceState { code: Some(16), .. }) => Ok(true),
        Some(state) => {
            let code = state.code.unwrap_or_default();
            let name = state.name.as_deref().unwrap_or_default();
            Err(InstanceError::IncorrectState(format!(
                "{} - {}",
                code, name
            )))
        }
        None => Err(InstanceError::IncorrectState("unknown".to_string())),
    }
}

pub fn has_security_group(instance: &Instance, sg_id: &str) -> Result<bool, InstanceError> {
    if let Some(sg_vec) = &instance.security_groups {
        if sg_vec
            .iter()
            .any(|x| x.group_id == Some(String::from(sg_id)))
        {
            return Ok(true);
        }
    }

    Err(InstanceError::SecurityGroupNotAttached)
}

pub fn get_public_ip(instance: &Instance) -> Result<IpAddr, InstanceError> {
    match &instance.public_ip_address {
        None => Err(InstanceError::NoPublicIP),
        Some(ip) => ip.parse().map_err(InstanceError::MalformedPublicIP),
    }
}

pub fn get_only_item<T>(item_vec: &Option<Vec<T>>) -> Result<&T, CardinalityError> {
    match item_vec {
        Some(item_vec) if item_vec.len() == 1 => Ok(&item_vec[0]),
        Some(item_vec) if item_vec.len() > 1 => Err(CardinalityError::TooMany),
        _ => Err(CardinalityError::None),
    }
}
