use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    RecvError,
    MaxConcurrent,
    MaxQueue,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RecvError => write!(f, "Unable to receive data from the channel"),
            Error::MaxConcurrent => write!(f, "Max number of tasks running at once"),
            Error::MaxQueue => write!(f, "Max number of tasks in queue"),
        }
    }
}

impl std::error::Error for Error {}
