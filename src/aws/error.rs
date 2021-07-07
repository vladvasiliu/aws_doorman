use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum AWSError {
    NothingToDo(String),
    CardinalityError(String),
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
            Self::NothingToDo(err) => write!(f, "{}", err),
        }
    }
}
