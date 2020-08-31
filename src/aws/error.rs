use core::fmt;
use std::error::Error;

use rusoto_core::RusotoError;
use rusoto_ec2::{
    DescribeManagedPrefixListsError, GetManagedPrefixListEntriesError, ModifyManagedPrefixListError,
};

#[derive(Debug)]
pub enum AWSError {
    CardinalityError(String),
    DescribeError(RusotoError<DescribeManagedPrefixListsError>),
    GetEntriesError(RusotoError<GetManagedPrefixListEntriesError>),
    ModifyError(RusotoError<ModifyManagedPrefixListError>),
}

impl Error for AWSError {}

impl fmt::Display for AWSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CardinalityError(msg) => write!(f, "{}", msg),
            Self::DescribeError(err) => write!(f, "{}", err),
            Self::GetEntriesError(err) => write!(f, "{}", err),
            Self::ModifyError(err) => write!(f, "{}", err),
        }
    }
}

impl From<RusotoError<DescribeManagedPrefixListsError>> for AWSError {
    fn from(err: RusotoError<DescribeManagedPrefixListsError>) -> Self {
        Self::DescribeError(err)
    }
}

impl From<RusotoError<GetManagedPrefixListEntriesError>> for AWSError {
    fn from(err: RusotoError<GetManagedPrefixListEntriesError>) -> Self {
        Self::GetEntriesError(err)
    }
}

impl From<RusotoError<ModifyManagedPrefixListError>> for AWSError {
    fn from(err: RusotoError<ModifyManagedPrefixListError>) -> Self {
        Self::ModifyError(err)
    }
}
