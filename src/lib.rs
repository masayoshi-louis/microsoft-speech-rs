extern crate failure;
#[macro_use]
extern crate failure_derive;

use speech_api::*;
use std::ffi;

mod speech_api;
pub mod speech;

#[derive(Fail, Debug)]
pub enum SpxError {
    #[fail(display = "Speech API error code: {}.", _0)]
    General(usize),
    #[fail(display = "Invalid CString (NulError).")]
    StrNulError(#[cause] ffi::NulError),
}

impl From<ffi::NulError> for SpxError {
    fn from(err: ffi::NulError) -> Self {
        return SpxError::StrNulError(err);
    }
}

#[inline(always)]
fn convert_err(hr: usize) -> Result<(), SpxError> {
    if hr != SPX_NOERROR {
        return Err(SpxError::General(hr));
    }
    return Ok(());
}
