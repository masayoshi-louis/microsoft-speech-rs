use std::borrow::Borrow;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::audio::AudioConfig;
use crate::convert_err;
use crate::recognizer::AbstractAsyncRecognizer;
use crate::recognizer::AsyncRecognizer;
use crate::recognizer::events::RecognitionCanceledEvent;
use crate::recognizer::events::RecognitionResultEvent;
use crate::recognizer::RecognitionResult;
use crate::recognizer::Recognizer;
use crate::speech_api::*;
use crate::SpeechConfig;
use crate::SpxError;
use crate::SPXHANDLE_INVALID;

type R = RecognitionResult;
type E = RecognitionResultEvent<R>;
type C = RecognitionCanceledEvent;

pub struct SpeechRecognizer {
    base: AbstractAsyncRecognizer<E, C>,
}

impl SpeechRecognizer {
    pub fn from_config(config: impl Borrow<SpeechConfig>, audio: Option<AudioConfig>) -> Result<SpeechRecognizer, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(
                recognizer_create_speech_recognizer_from_config(
                    &mut handle,
                    config.borrow().get_handle(),
                    audio.as_ref().map(|c| c.get_handle()).unwrap_or(0 as SPXAUDIOCONFIGHANDLE),
                )
            )?;
        }
        Ok(SpeechRecognizer {
            base: AbstractAsyncRecognizer::create(handle)?,
        })
    }
}

impl Deref for SpeechRecognizer {
    type Target = dyn AsyncRecognizer<R, E, C, Target=dyn Recognizer>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for SpeechRecognizer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
