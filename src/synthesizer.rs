use crate::{FromHandle, SmartHandle, SpxError, convert_err, FfiObject};
use crate::speech_api::*;

pub struct SpeechSynthesisResult {
    handle: SmartHandle<SPXRESULTHANDLE>,
}

impl SpeechSynthesisResult {
    pub fn audio_length(&self) -> Result<u32, SpxError> {
        let mut len = 0;
        unsafe {
            convert_err(synth_result_get_audio_length(self.get_handle(), &mut len))?
        }
        Ok(len)
    }

    pub fn audio_data(&self) -> Result<Vec<u8>, SpxError> {
        let buff = FfiObject::new_uninitialized(self.audio_length()? as usize);
        let mut filled_size = 0;
        unsafe {
            convert_err(synth_result_get_audio_data(
                self.get_handle(),
                buff.ptr,
                buff.size as u32,
                &mut filled_size,
            ))?
        }
        Ok(buff.into_vec(filled_size as usize))
    }

    #[inline(always)]
    pub fn get_handle(&self) -> SPXRESULTHANDLE {
        self.handle.get()
    }
}

impl FromHandle<SPXRESULTHANDLE, SpxError> for SpeechSynthesisResult {
    fn from_handle(handle: SPXRESULTHANDLE) -> Result<Self, SpxError> {
        Ok(SpeechSynthesisResult {
            handle: SmartHandle::create(
                "SpeechSynthesisResult",
                handle,
                synthesizer_result_handle_release,
            )
        })
    }
}