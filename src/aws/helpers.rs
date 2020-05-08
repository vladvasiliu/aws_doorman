use rusoto_ec2::{Instance, InstanceState, SecurityGroup};

use crate::aws::error::{CardinalityError, InstanceError};
use std::net::IpAddr;
use crate::aws::IPRule;

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

/// Returns a Vec containing the IP addresses of the AWS Security Group if the rule we want
/// to add is present
///
/// An AWS Security Group Rule is identified by its ports and protocols.
fn ips_for_rule_in_sg(rule: IPRule, sg: &SecurityGroup) -> Vec<&str> {
    sg.ip_permissions.as_ref().map_or_else(Vec::new, |ip_permission_vec| {
        ip_permission_vec.iter()
            .filter(|ip_permission|{rule == **ip_permission})
            .flat_map(|ip_permission| {
                ip_permission.ip_ranges.as_ref().map_or_else(Vec::new, |ip_range_vec| {
                    ip_range_vec.iter().filter_map(|ip_range| {
                        if ip_range.description.as_ref() == Some(&rule.id) {
                            ip_range.cidr_ip.as_deref()
                        } else {
                            None
                        }
                    }).collect()
                })
            }).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ips_for_rule_in_sg {
        use super::*;
        use rusoto_ec2::{IpPermission};

        #[test]
        fn returns_empty_vec_for_sg_with_none_permissions() {
            let sg = SecurityGroup{
                ip_permissions: None,
                ..Default::default()
            };
            let rule: IPRule = Default::default();
            let ip_vec = ips_for_rule_in_sg(rule, &sg);
            assert!(ip_vec.is_empty())
        }

        #[test]
        fn returns_empty_vec_for_sg_with_empty_permissions() {
            let sg = SecurityGroup{
                ip_permissions: Some(vec![]),
                ..Default::default()
            };
            let rule: IPRule = Default::default();
            let ip_vec = ips_for_rule_in_sg(rule, &sg);
            assert!(ip_vec.is_empty())
        }

        #[test]
        fn returns_empty_vec_for_sg_with_different_permissions() {
            let ip_permission = IpPermission {
                from_port: Some(10),
                to_port: Some(10),
                ip_protocol: Some("tcp".into()),
                ..Default::default()
            };
            let sg = SecurityGroup{
                ip_permissions: Some(vec![ip_permission]),
                ..Default::default()
            };
            let rule: IPRule = Default::default();
            let ip_vec = ips_for_rule_in_sg(rule, &sg);
            assert!(ip_vec.is_empty())
        }
    }
}
