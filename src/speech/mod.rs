use convert_err;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ffi;
use std::ffi::CString;
use SmartHandle;

pub mod audio;
pub mod recognizer;

#[derive(Debug)]
pub struct SpeechConfig {
    handle: SmartHandle<SPXSPEECHCONFIGHANDLE>,
}

impl SpeechConfig {
    pub fn from_subscription<S1, S2>(subscription: S1, region: S2) -> Result<SpeechConfig, SpxError>
        where S1: Into<Vec<u8>>, S2: Into<Vec<u8>> {
        let c_sub = CString::new(subscription)?;
        let c_region = CString::new(region)?;
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(speech_config_from_subscription(&mut handle, c_sub.as_ptr(), c_region.as_ptr()))?;
        }
        let mut result = SpeechConfig {
            handle: SmartHandle::create(handle, speech_config_release),
        };

        Ok(result)
    }

    pub fn get_handle(&self) -> SPXSPEECHCONFIGHANDLE {
        self.handle.get()
    }
}
