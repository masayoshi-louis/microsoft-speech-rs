use convert_err;
use SmartHandle;
use speech_api::*;
use SpxError;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;

const NULL_C_STR_PTR: *const c_char = 0 as *const c_char;

pub enum PropertyId
{
    SpeechServiceConnectionKey = 1000,
    SpeechServiceConnectionEndpoint = 1001,
    SpeechServiceConnectionRegion = 1002,
    SpeechServiceAuthorizationToken = 1003,
    SpeechServiceAuthorizationType = 1004,
    SpeechServiceConnectionEndpointId = 1005,

    SpeechServiceConnectionTranslationToLanguages = 2000,
    SpeechServiceConnectionTranslationVoice = 2001,
    SpeechServiceConnectionTranslationFeatures = 2002,
    SpeechServiceConnectionIntentRegion = 2003,

    SpeechServiceConnectionRecoMode = 3000,
    SpeechServiceConnectionRecoLanguage = 3001,
    SpeechSessionId = 3002,

    SpeechServiceResponseRequestDetailedResultTrueFalse = 4000,
    SpeechServiceResponseRequestProfanityFilterTrueFalse = 4001,

    SpeechServiceResponseJsonResult = 5000,
    SpeechServiceResponseJsonErrorDetails = 5001,

    CancellationDetailsReason = 6000,
    CancellationDetailsReasonText = 6001,
    CancellationDetailsReasonDetailedText = 6002,

    LanguageUnderstandingServiceResponseJsonResult = 7000,
}

#[derive(Debug)]
pub struct PropertyBag {
    handle: SmartHandle<SPXPROPERTYBAGHANDLE>,
}

impl PropertyBag {
    #[inline(always)]
    pub(crate)
    fn create(hcfg: SPXHANDLE,
              f: unsafe extern "C" fn(SPXHANDLE, *mut SPXPROPERTYBAGHANDLE) -> SPXHR) -> Result<PropertyBag, SpxError> {
        let handle = ::spx_populate(hcfg, f)?;
        Ok(PropertyBag {
            handle: SmartHandle::create("PropertyBag", handle, property_bag_release),
        })
    }

    #[inline]
    pub fn get_handle(&self) -> SPXPROPERTYBAGHANDLE {
        self.handle.get()
    }

    pub fn get(&self, id: PropertyId) -> Result<Option<String>, SpxError> {
        unsafe {
            let ret = property_bag_get_string(self.get_handle(), id as i32, NULL_C_STR_PTR, NULL_C_STR_PTR);
            return if ret == NULL_C_STR_PTR {
                Ok(None)
            } else {
                Ok(Some(CStr::from_ptr(ret).to_string_lossy().into_owned()))
            };
        }
    }

    pub fn set<T: AsRef<str>>(&mut self, id: PropertyId, v: T) -> Result<(), SpxError> {
        let s = CString::new(v.as_ref())?;
        unsafe {
            convert_err(property_bag_set_string(self.get_handle(), id as i32, NULL_C_STR_PTR, s.as_ptr()))?;
        }
        Ok(())
    }
}
