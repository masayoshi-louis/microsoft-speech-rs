extern crate failure;
#[macro_use]
extern crate failure_derive;

use speech_api::*;
use std::ffi;
use std::ffi::CString;

mod speech_api;

#[derive(Fail, Debug)]
pub enum SpxError {
    #[fail(display = "Speech API error code: {}.", _0)]
    General(usize),
    #[fail(display = "Invalid CString (NulError).")]
    StrNulError(#[cause] ffi::NulError),
}

pub struct SpeechConfig {
    handle: SPXSPEECHCONFIGHANDLE,
}

impl SpeechConfig {
    pub fn from_subscription<S1, S2>(subscription: S1, region: S2) -> Result<SpeechConfig, SpxError>
        where S1: Into<Vec<u8>>, S2: Into<Vec<u8>> {
        let c_sub = CString::new(subscription)?;
        let c_region = CString::new(region)?;
        let mut result = SpeechConfig {
            handle: 0 as SPXSPEECHCONFIGHANDLE,
        };
        unsafe {
            check_err(speech_config_from_subscription(&mut result.handle, c_sub.as_ptr(), c_region.as_ptr()))?;
        }
        return Ok(result);
    }
}

impl From<ffi::NulError> for SpxError {
    fn from(err: ffi::NulError) -> Self {
        return SpxError::StrNulError(err);
    }
}

fn check_err(hr: usize) -> Result<(), SpxError> {
    if hr != SPX_NOERROR {
        return Err(SpxError::General(hr));
    }
    return Ok(());
}
