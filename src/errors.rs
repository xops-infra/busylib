use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct DecryptError {
    pub(crate) details: String,
}

impl Error for DecryptError {}

impl Display for DecryptError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

#[derive(Debug)]
pub struct RemoveFilesError {
    pub(crate) details: String,
}

impl Error for RemoveFilesError {}

impl Display for RemoveFilesError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl From<tokio_cron_scheduler::JobSchedulerError> for RemoveFilesError {
    fn from(error: tokio_cron_scheduler::JobSchedulerError) -> Self {
        Self {
            details: error.to_string(),
        }
    }
}
