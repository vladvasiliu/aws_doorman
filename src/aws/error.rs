use std::error::Error;
use std::net::AddrParseError;
use std::{fmt, fmt::Formatter};

use rusoto_core::request::BufferedHttpResponse;
use rusoto_core::RusotoError;
use rusoto_ec2::{
    AuthorizeSecurityGroupIngressError, DescribeInstancesError, DescribeSecurityGroupsError,
};

#[derive(Debug)]
pub enum InstanceError {
    ReturnedNone,
    ReturnedTooMany,
    NoPublicIP,
    MalformedPublicIP(AddrParseError),
    SecurityGroupNotAttached,
    IncorrectState(String),
    UnknownError(RusotoError<DescribeInstancesError>),
}

impl Error for InstanceError {}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReturnedNone => write!(f, "no instances returned"),
            Self::ReturnedTooMany => write!(f, "too many instances returned"),
            Self::NoPublicIP => write!(f, "no public ip"),
            Self::MalformedPublicIP(err) => write!(f, "malformed public IP: {}", err),
            Self::SecurityGroupNotAttached => write!(f, "requested security group is not attached"),
            Self::IncorrectState(err) => write!(f, "incorrect state: {}", err),
            Self::UnknownError(err) => write!(f, "unknown error: {}", err),
        }
    }
}

impl From<CardinalityError> for InstanceError {
    fn from(err: CardinalityError) -> Self {
        match err {
            CardinalityError::TooMany => Self::ReturnedTooMany,
            CardinalityError::None => Self::ReturnedNone,
        }
    }
}

impl From<RusotoError<DescribeInstancesError>> for InstanceError {
    fn from(err: RusotoError<DescribeInstancesError>) -> Self {
        Self::UnknownError(err)
    }
}

#[derive(Debug)]
pub enum SGAuthorizeIngressError {
    DuplicateRule(HttpResponseDescription),
    MalformedRule(HttpResponseDescription),
    RuleLimitExceeded(HttpResponseDescription),
    UnknownHttpError(HttpResponseDescription),
    Unknown(RusotoError<AuthorizeSecurityGroupIngressError>),
}

impl Error for SGAuthorizeIngressError {}

impl fmt::Display for SGAuthorizeIngressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRule(err) => write!(f, "{}", err),
            Self::MalformedRule(err) => write!(f, "{}", err),
            Self::RuleLimitExceeded(err) => write!(f, "{}", err),
            Self::UnknownHttpError(err) => write!(f, "{}", err),
            Self::Unknown(err) => write!(f, "Unknown error: {}", err),
        }
    }
}

impl From<RusotoError<AuthorizeSecurityGroupIngressError>> for SGAuthorizeIngressError {
    fn from(err: RusotoError<AuthorizeSecurityGroupIngressError>) -> Self {
        match err {
            RusotoError::Unknown(http_response) => {
                Self::from(HttpResponseDescription::from(http_response))
            }
            _ => Self::Unknown(err),
        }
    }
}

impl From<HttpResponseDescription> for SGAuthorizeIngressError {
    fn from(err: HttpResponseDescription) -> Self {
        // If status is 403, it should be handled by a more generic error
        // If status is other than 400, we don't know what it is
        if err.status != 400 {
            return Self::UnknownHttpError(err);
        }

        // If there is no error, we can't do anything
        // There shouldn't be several errors
        // if there are, the first one is probably the one we want
        let error_detail = match err.errors.first() {
            None => return Self::UnknownHttpError(err),
            Some(error_detail) => error_detail,
        };

        match error_detail.code.as_deref() {
            Some("InvalidPermission.Duplicate") => Self::DuplicateRule(err),
            Some("InvalidPermission.Malformed") => Self::MalformedRule(err),
            Some("RulesPerSecurityGroupLimitExceeded") => return Self::RuleLimitExceeded(err),
            _ => Self::UnknownHttpError(err),
        }
    }
}

#[derive(Debug)]
pub enum SecurityGroupError {
    ReturnedTooMany,
    ReturnedNone,
    DescribeError(HttpResponseDescription),
    AuthorizeIngressError(SGAuthorizeIngressError),
    NotFound(HttpResponseDescription),
    UnknownError(Box<dyn Error>),
}

impl Error for SecurityGroupError {}

impl fmt::Display for SecurityGroupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReturnedNone => write!(f, "no instances returned"),
            Self::ReturnedTooMany => write!(f, "too many instances returned"),
            Self::DescribeError(err) => write!(f, "failed to get security group: {}", err),
            Self::AuthorizeIngressError(err) => write!(f, "failed to authorize ip: {}", err),
            Self::NotFound(err) => write!(f, "{}", err),
            Self::UnknownError(err) => write!(f, "{}", err),
        }
    }
}

impl From<CardinalityError> for SecurityGroupError {
    fn from(err: CardinalityError) -> Self {
        match err {
            CardinalityError::TooMany => Self::ReturnedTooMany,
            CardinalityError::None => Self::ReturnedNone,
        }
    }
}

impl From<RusotoError<DescribeSecurityGroupsError>> for SecurityGroupError {
    fn from(err: RusotoError<DescribeSecurityGroupsError>) -> Self {
        Self::UnknownError(err.into())
    }
}

impl From<RusotoError<AuthorizeSecurityGroupIngressError>> for SecurityGroupError {
    fn from(err: RusotoError<AuthorizeSecurityGroupIngressError>) -> Self {
        Self::AuthorizeIngressError(err.into())
    }
}

#[derive(Debug)]
pub enum AWSClientError<E> {
    Service(E),
    PermissionDenied(HttpResponseDescription),
    Unknown(Box<dyn Error>),
}

impl<E: Error + 'static> Error for AWSClientError<E> {}

impl<E: Error + 'static> fmt::Display for AWSClientError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Service(err) => write!(f, "{}", err),
            Self::PermissionDenied(err) => write!(f, "{}", err),
            Self::Unknown(err) => write!(f, "{}", err),
        }
    }
}

impl<E: Error + 'static, F: std::convert::From<RusotoError<E>>> From<RusotoError<E>>
    for AWSClientError<F>
{
    fn from(err: RusotoError<E>) -> Self {
        match err {
            RusotoError::Unknown(http_resp) if http_resp.status == 403 => {
                Self::PermissionDenied(http_resp.into())
            }
            RusotoError::Unknown(_) | RusotoError::Service(_) => Self::Service(err.into()),
            _ => Self::Unknown(err.into()),
        }
    }
}

impl From<InstanceError> for AWSClientError<InstanceError> {
    fn from(err: InstanceError) -> Self {
        Self::Service(err)
    }
}

impl From<SecurityGroupError> for AWSClientError<SecurityGroupError> {
    fn from(err: SecurityGroupError) -> Self {
        Self::Service(err)
    }
}

impl From<AddrParseError> for InstanceError {
    fn from(err: AddrParseError) -> Self {
        Self::MalformedPublicIP(err)
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

#[derive(Debug)]
pub enum CardinalityError {
    None,
    TooMany,
}

impl Error for CardinalityError {}

impl fmt::Display for CardinalityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None found"),
            Self::TooMany => write!(f, "Too many"),
        }
    }
}

pub type SGClientResult<T> = Result<T, AWSClientError<SecurityGroupError>>;
