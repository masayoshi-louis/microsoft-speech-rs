use convert_err;
use PropertyBag;
use SmartHandle;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ffi::CString;
use std::ops::Deref;
use std::ops::DerefMut;


#[derive(Debug)]
pub struct SpeechConfig {
    handle: SmartHandle<SPXSPEECHCONFIGHANDLE>,
    props: PropertyBag,
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
        let result = SpeechConfig {
            handle: SmartHandle::create("SpeechConfig", handle, speech_config_release),
            props: PropertyBag::create(handle, speech_config_get_property_bag)?,
        };
        Ok(result)
    }

    #[inline]
    pub fn get_handle(&self) -> SPXSPEECHCONFIGHANDLE {
        self.handle.get()
    }
}

impl Deref for SpeechConfig {
    type Target = PropertyBag;

    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl DerefMut for SpeechConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.props
    }
}

impl AsRef<SpeechConfig> for SpeechConfig {
    fn as_ref(&self) -> &SpeechConfig {
        self
    }
}
