use rusoto_ec2::SecurityGroup;

use crate::aws::error::CardinalityError;
use crate::aws::IPRule;

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
/// IpAddr doesn't have a netmask, so this function has to return a str
pub fn ips_for_rule_in_sg<'a>(rule: &IPRule, sg: &'a SecurityGroup) -> Vec<&'a str> {
    sg.ip_permissions
        .as_ref()
        .map_or_else(Vec::new, |ip_permission_vec| {
            ip_permission_vec
                .iter()
                .filter(|ip_permission| rule == *ip_permission)
                .flat_map(|ip_permission| {
                    ip_permission
                        .ip_ranges
                        .as_ref()
                        .map_or_else(Vec::new, |ip_range_vec| {
                            ip_range_vec
                                .iter()
                                .filter_map(|ip_range| {
                                    if ip_range.description.as_ref() == Some(&rule.id) {
                                        ip_range.cidr_ip.as_deref()
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        })
                })
                .collect()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ips_for_rule_in_sg {
        use super::*;
        use rusoto_ec2::{IpPermission, IpRange};

        #[test]
        fn returns_empty_vec_for_sg_with_none_permissions() {
            let sg = SecurityGroup {
                ip_permissions: None,
                ..Default::default()
            };
            let rule: IPRule = Default::default();
            let ip_vec = ips_for_rule_in_sg(&rule, &sg);
            assert!(ip_vec.is_empty())
        }

        #[test]
        fn returns_empty_vec_for_sg_with_empty_permissions() {
            let sg = SecurityGroup {
                ip_permissions: Some(vec![]),
                ..Default::default()
            };
            let rule: IPRule = Default::default();
            let ip_vec = ips_for_rule_in_sg(&rule, &sg);
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
            let sg = SecurityGroup {
                ip_permissions: Some(vec![ip_permission]),
                ..Default::default()
            };
            let rule: IPRule = Default::default();
            let ip_vec = ips_for_rule_in_sg(&rule, &sg);
            assert!(ip_vec.is_empty())
        }

        #[test]
        fn returns_ips_for_sg_with_multiple_ips_and_correct_description() {
            let ip_rule_id = String::from("some description");
            let from_port: i64 = 10;
            let to_port: i64 = 10;
            let ip_protocol = String::from("tcp");

            let ip_vec = vec![String::from("1.1.1.1/32"), String::from("2.2.2.2/32")];

            let ip_ranges = ip_vec
                .iter()
                .map(|ip| IpRange {
                    cidr_ip: Some(ip.to_owned()),
                    description: Some(ip_rule_id.to_owned()),
                })
                .collect();

            let ip_permission = IpPermission {
                from_port: Some(from_port),
                to_port: Some(to_port),
                ip_protocol: Some(ip_protocol.to_owned()),
                ip_ranges: Some(ip_ranges),
                ..Default::default()
            };
            let sg = SecurityGroup {
                ip_permissions: Some(vec![ip_permission]),
                ..Default::default()
            };
            let rule = IPRule {
                id: ip_rule_id.to_owned(),
                from_port,
                to_port,
                ip_protocol: ip_protocol.to_owned(),
                ..Default::default()
            };
            let res = ips_for_rule_in_sg(&rule, &sg);
            assert_eq!(ip_vec, res)
        }
    }

    mod get_only_item {
        use crate::aws::error::CardinalityError;
        use crate::aws::helpers::get_only_item;

        #[test]
        fn error_for_none() {
            let item_vec: Option<Vec<i32>> = None;
            let res = get_only_item(&item_vec);
            assert_eq!(res, Err(CardinalityError::None))
        }

        #[test]
        fn error_for_empty_vec() {
            let item_vec: Option<Vec<i32>> = Some(vec![]);
            let res = get_only_item(&item_vec);
            assert_eq!(res, Err(CardinalityError::None))
        }

        #[test]
        fn error_for_too_many() {
            let item_vec: Option<Vec<i32>> = Some(vec![1, 2]);
            let res = get_only_item(&item_vec);
            assert_eq!(res, Err(CardinalityError::TooMany))
        }

        #[test]
        fn ok_for_single_element() {
            let item_vec: Option<Vec<i32>> = Some(vec![1]);
            let res = get_only_item(&item_vec);
            assert_eq!(res, Ok(&1))
        }
    }
}
