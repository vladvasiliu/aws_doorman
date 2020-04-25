use rusoto_ec2::{Instance, InstanceState};

use crate::aws::error::EC2InstanceError;
use std::net::IpAddr;

pub(in crate::aws) fn is_running(instance: &Instance) -> Result<bool, EC2InstanceError> {
    match &instance.state {
        Some(InstanceState { code: Some(16), .. }) => Ok(true),
        Some(state) => {
            let code = state.code.unwrap_or_default();
            let name = state.name.as_deref().unwrap_or_default();
            Err(EC2InstanceError::InstanceHasIncorrectState(format!(
                "{} - {}",
                code, name
            )))
        }
        None => Err(EC2InstanceError::InstanceHasIncorrectState(
            "unknown".to_string(),
        )),
    }
}

pub(in crate::aws) fn has_security_group(
    instance: &Instance,
    sg_id: &str,
) -> Result<bool, EC2InstanceError> {
    if let Some(sg_vec) = &instance.security_groups {
        if sg_vec
            .iter()
            .any(|x| x.group_id == Some(String::from(sg_id)))
        {
            return Ok(true);
        }
    }

    Err(EC2InstanceError::SecurityGroupNotAttached)
}

pub(in crate::aws) fn get_public_ip(instance: &Instance) -> Result<IpAddr, EC2InstanceError> {
    match &instance.public_ip_address {
        None => Err(EC2InstanceError::InstanceHasNoPublicIP),
        Some(ip) => ip
            .parse()
            .map_err(EC2InstanceError::InstanceHasMalformedPublicIP),
    }
}
