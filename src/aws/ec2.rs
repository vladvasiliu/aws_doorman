use std::{fmt, net::IpAddr, result::Result};

use rusoto_core::request::BufferedHttpResponse;
use rusoto_core::RusotoError;
use rusoto_ec2::{DescribeInstancesError, DescribeInstancesRequest, Ec2, Ec2Client};
use std::fmt::Formatter;

pub struct EC2Instance {
    pub id: String,
    pub name: Option<String>,
    pub ip_addresses: Vec<IpAddr>,
    pub running: bool,
}

impl EC2Instance {
    pub async fn from_query(id: String, ec2_client: Ec2Client) -> Result<(), EC2InstanceError> {
        let dir = DescribeInstancesRequest {
            instance_ids: Some(vec![id]),
            ..Default::default()
        };
        let res = ec2_client.describe_instances(dir).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum EC2InstanceError {
    DescribeInstancesPermissionDenied(HttpResponseDescription),
    DescribeInstancesBadRequest(HttpResponseDescription),
    DescribeInstancesUnknownError(RusotoError<DescribeInstancesError>),
}

impl fmt::Display for EC2InstanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DescribeInstancesPermissionDenied(err)
            | Self::DescribeInstancesBadRequest(err) => write!(f, "Failed to get instance information: {}", err),
            Self::DescribeInstancesUnknownError(err) => {
                write!(f, "Unknown error occurred. Cause: {:#?}", err)
            }
        }
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
        let mut msg  = String::from("Request failed.");
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
            let code = error.descendants().find(|n| n.tag_name() == "Code".into()).unwrap().text().map(String::from);
            let message = error.descendants().find(|n| n.tag_name() == "Message".into()).unwrap().text().map(String::from);
            let hre = HttpResponseError {code, message};
            hre_vec.push(hre);
        }
        Self {
            status: hrd.status.as_u16(),
            errors: hre_vec,
            source: hrd,
        }
    }
}
