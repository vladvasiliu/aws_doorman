use std::{fmt, net::IpAddr, result::Result};

use rusoto_core::request::BufferedHttpResponse;
use rusoto_core::RusotoError;
use rusoto_ec2::{
    DescribeInstancesError, DescribeInstancesRequest, Ec2, Ec2Client, Instance, InstanceState,
    Reservation,
};
use std::fmt::Formatter;
use std::net::AddrParseError;

#[derive(Debug)]
pub struct EC2Instance {
    pub id: String,
    pub name: Option<String>,
    pub ip_addresses: Vec<IpAddr>,
    pub running: bool,
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

        let running = match &instance.state {
            Some(InstanceState { code: Some(16), .. }) => true,
            Some(state) => {
                let code = state.code.unwrap_or_default();
                let name = state.name.as_deref().unwrap_or_default();
                return Err(EC2InstanceError::InstanceHasIncorrectState(format!(
                    "{} - {}",
                    code, name
                )));
            }
            None => {
                return Err(EC2InstanceError::InstanceHasIncorrectState(
                    "unknown".to_string(),
                ))
            }
        };

        let public_ip: IpAddr = match &instance.public_ip_address {
            None => return Err(EC2InstanceError::InstanceHasNoPublicIP),
            Some(ip) => ip.parse()?,
        };

        Ok(Self {
            id,
            name: None,
            ip_addresses: vec![public_ip],
            running,
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

#[derive(Debug)]
pub enum EC2InstanceError {
    DescribeInstancesPermissionDenied(HttpResponseDescription),
    DescribeInstancesBadRequest(HttpResponseDescription),
    DescribeInstancesUnknownError(RusotoError<DescribeInstancesError>),
    DescribeInstancesReturnedNone,
    DescribeInstancesReturnedTooMany,
    InstanceHasNoPublicIP,
    InstanceHasMalformedPublicIP(AddrParseError),
    InstanceHasIncorrectState(String),
}

impl fmt::Display for EC2InstanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut msg = String::from("Failed to get instance information: ");
        match self {
            Self::DescribeInstancesPermissionDenied(err)
            | Self::DescribeInstancesBadRequest(err) => {
                msg.push_str(&format!("{}", err));
            }
            Self::DescribeInstancesReturnedNone => msg.push_str("no instances returned"),
            Self::DescribeInstancesReturnedTooMany => msg.push_str("too many instances returned"),
            Self::DescribeInstancesUnknownError(err) => {
                msg.push_str(&format!("Unknown error occurred. Cause: {:#?}", err));
            }
            Self::InstanceHasNoPublicIP => msg.push_str("Instance has no public IP"),
            Self::InstanceHasMalformedPublicIP(err) => {
                msg.push_str(&format!("Public IP is malformed: {}", err))
            }
            Self::InstanceHasIncorrectState(err) => {
                msg.push_str(&format!("Incorrect state: {}", err))
            }
        }
        write!(f, "{}", msg)
    }
}

impl From<RusotoError<DescribeInstancesError>> for EC2InstanceError {
    fn from(err: RusotoError<DescribeInstancesError>) -> Self {
        match err {
            RusotoError::Unknown(http_resp) if http_resp.status == 400 => {
                Self::DescribeInstancesBadRequest(http_resp.into())
            }
            RusotoError::Unknown(http_resp) if http_resp.status == 403 => {
                Self::DescribeInstancesPermissionDenied(http_resp.into())
            }
            _ => Self::DescribeInstancesUnknownError(err),
        }
    }
}

impl From<AddrParseError> for EC2InstanceError {
    fn from(err: AddrParseError) -> Self {
        Self::InstanceHasMalformedPublicIP(err)
    }
}

#[derive(Debug)]
struct HttpResponseError {
    code: Option<String>,
    message: Option<String>,
}

#[derive(Debug)]
pub struct HttpResponseDescription {
    status: u16,
    errors: Vec<HttpResponseError>,
    source: BufferedHttpResponse,
}

impl fmt::Display for HttpResponseDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut msg = String::from("Request failed.");
        for error in &self.errors {
            let code = error.code.as_deref().unwrap_or_default();
            let message = error.message.as_deref().unwrap_or_default();
            msg.push_str(format!(" {} - {}", code, message).as_str())
        }
        write!(f, "{}", msg)
    }
}

impl From<BufferedHttpResponse> for HttpResponseDescription {
    fn from(hrd: BufferedHttpResponse) -> Self {
        let doc = String::from_utf8(hrd.body.to_vec()).unwrap();
        let xml_doc = roxmltree::Document::parse(&doc).unwrap();
        let errors = xml_doc
            .descendants()
            .find(|n| n.tag_name() == "Errors".into())
            .unwrap();
        let mut hre_vec = vec![];
        for error in errors.children() {
            let code = error
                .descendants()
                .find(|n| n.tag_name() == "Code".into())
                .unwrap()
                .text()
                .map(String::from);
            let message = error
                .descendants()
                .find(|n| n.tag_name() == "Message".into())
                .unwrap()
                .text()
                .map(String::from);
            let hre = HttpResponseError { code, message };
            hre_vec.push(hre);
        }
        Self {
            status: hrd.status.as_u16(),
            errors: hre_vec,
            source: hrd,
        }
    }
}
