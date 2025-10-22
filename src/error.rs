use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct Invalid;

impl Display for Invalid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid")
    }
}

impl Error for Invalid {}
