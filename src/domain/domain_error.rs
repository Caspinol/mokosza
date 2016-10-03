extern crate curl;

use std::fmt;
use std::error;
/* 
Fetching error stuct definition
 */

#[derive(Debug)]
pub enum DomainError {
    SkippedURL,
    RobotsError,
    InvalidURL,
    FetchError(curl::Error),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DomainError::SkippedURL => write!(f, "URL skipped"),
            DomainError::RobotsError => write!(f, "robot.txt missing or malformed"),
            DomainError::InvalidURL => write!(f, "Malformed url"),
            DomainError::FetchError(ref err) => write!(f, "Failed to grab domain: {}", err),
        }
    }
}

impl error::Error for DomainError {
    fn description(&self) -> &str {
        match *self {
            DomainError::SkippedURL => "URL was skipped due to robots.txt policy.",
            DomainError::RobotsError => "robots.txt file missing or malformed.",
            DomainError::InvalidURL => "Specified URL is invalid/malformed",
            DomainError::FetchError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DomainError::SkippedURL => None,
            DomainError::RobotsError => None,
            DomainError::InvalidURL => None,
            DomainError::FetchError(ref err) => err.cause(),
        }
    }
}

impl From<curl::Error> for DomainError {
    fn from(err: curl::Error) -> DomainError {
        DomainError::FetchError(err)
    }
}
