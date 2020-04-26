use std::error::Error;
use std::net::AddrParseError;
use std::{fmt, fmt::Formatter};

use rusoto_core::credential::CredentialsError;
use rusoto_core::request::BufferedHttpResponse;
use rusoto_core::RusotoError;
use rusoto_ec2::DescribeInstancesError;

#[derive(Debug)]
pub enum AWSClientError {
    CredentialsError(CredentialsError),
    DescribeInstancesPermissionDenied(HttpResponseDescription),
    DescribeInstancesBadRequest(HttpResponseDescription),
    DescribeInstancesUnknownError(RusotoError<DescribeInstancesError>),
    DescribeInstancesReturnedNone,
    DescribeInstancesReturnedTooMany,
    InstanceHasNoPublicIP,
    InstanceHasMalformedPublicIP(AddrParseError),
    InstanceHasIncorrectState(String),
    SecurityGroupNotAttached,
}

impl Error for AWSClientError {}

impl fmt::Display for AWSClientError {
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
            Self::SecurityGroupNotAttached => {
                msg.push_str("The requested security group is not attached")
            }
            Self::CredentialsError(err) => {
                msg.push_str(&format!("Credentials error: {}", err));
            }
        }
        write!(f, "{}", msg)
    }
}

impl From<RusotoError<DescribeInstancesError>> for AWSClientError {
    fn from(err: RusotoError<DescribeInstancesError>) -> Self {
        match err {
            RusotoError::Unknown(http_resp) if http_resp.status == 400 => {
                Self::DescribeInstancesBadRequest(http_resp.into())
            }
            RusotoError::Unknown(http_resp) if http_resp.status == 403 => {
                Self::DescribeInstancesPermissionDenied(http_resp.into())
            }
            RusotoError::Credentials(err) => Self::CredentialsError(err),
            _ => Self::DescribeInstancesUnknownError(err),
        }
    }
}

impl From<AddrParseError> for AWSClientError {
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

// Needed because Rusoto always returns an Unknown error.
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
