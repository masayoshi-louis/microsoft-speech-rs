use convert_err;
use speech_api::*;
use SpxError;
use std::ffi;
use std::ffi::CString;

pub mod audio;

#[derive(Debug)]
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
            convert_err(speech_config_from_subscription(&mut result.handle, c_sub.as_ptr(), c_region.as_ptr()))?;
        }
        Ok(result)
    }
}

impl Drop for SpeechConfig {
    fn drop(&mut self) {
        unsafe {
            speech_config_release(self.handle);
        }
    }
}
