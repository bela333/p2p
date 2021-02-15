use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

macro_rules! implement_error {
    ($t:ty, $message:expr) => {
        impl From<$t> for Error {
            fn from(e: $t) -> Self {
                Self {
                    message: format!(concat!($message, ": {}"), e),
                }
            }
        }
    };
}

implement_error!(bs58::decode::Error, "Base58 error");
implement_error!(std::io::Error, "IO error");
implement_error!(std::array::TryFromSliceError, "Array conversion error");

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::error::Error for Error {}
