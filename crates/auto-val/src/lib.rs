mod value;

pub use value::*;

mod to_value;
pub use to_value::ToAutoValue;

mod types;
pub use types::*;

mod string;
pub use string::*;

mod owned_str;
pub use owned_str::*;

mod str_slice;
pub use str_slice::*;

mod linear;
pub use linear::*;

mod shared;
pub use shared::*;

mod array;
pub use array::*;

mod pair;
pub use pair::*;

mod obj;
pub use obj::*;

mod meta;
pub use meta::*;

mod kids;
pub use kids::*;

mod node;
pub use node::*;

mod path;
pub use path::*;

pub type AutoError = Box<dyn std::error::Error>;
pub type AutoResult<T> = Result<T, AutoError>;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IOError {
    #[error("IO error occurred with `{}`: {}", .src, .err)]
    FileError { err: std::io::Error, src: AutoStr },
    #[error("Error occurred `{}` -> `{}`: {}", .src, .dst, .err)]
    DualError {
        src: AutoStr,
        dst: AutoStr,
        err: std::io::Error,
    },
}

pub enum CommonResult<T> {
    Ok(T),
    Err(IOError),
}

impl From<std::io::Error> for IOError {
    fn from(err: std::io::Error) -> Self {
        IOError::FileError {
            err,
            src: AutoStr::new(),
        }
    }
}

impl IOError {
    pub fn file(err: std::io::Error, src: impl Into<AutoStr>) -> Self {
        IOError::FileError {
            err,
            src: src.into(),
        }
    }

    pub fn dual(err: std::io::Error, src: impl Into<AutoStr>, dst: impl Into<AutoStr>) -> Self {
        IOError::DualError {
            src: src.into(),
            dst: dst.into(),
            err: err,
        }
    }
}

impl<T> From<std::io::Result<T>> for CommonResult<T> {
    fn from(result: std::io::Result<T>) -> Self {
        match result {
            Ok(value) => CommonResult::Ok(value),
            Err(err) => CommonResult::Err(IOError::from(err)),
        }
    }
}

impl From<IOError> for CommonResult<()> {
    fn from(err: IOError) -> Self {
        CommonResult::Err(err)
    }
}

impl<T> From<CommonResult<T>> for AutoResult<T> {
    fn from(result: CommonResult<T>) -> Self {
        match result {
            CommonResult::Ok(value) => AutoResult::Ok(value),
            CommonResult::Err(err) => AutoResult::Err(err.into()),
        }
    }
}
