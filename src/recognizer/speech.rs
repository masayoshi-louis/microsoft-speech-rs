use audio::AudioConfig;
use audio::AudioInputStream;
use convert_err;
use recognizer::AbstractAsyncRecognizer;
use recognizer::AsyncRecognizer;
use recognizer::events::RecognitionCanceledEvent;
use recognizer::events::RecognitionResultEvent;
use recognizer::Recognizer;
use speech_api::*;
use SpeechConfig;
use SpxError;
use SPXHANDLE_INVALID;
use std::borrow::Borrow;
use std::ops::Deref;
use std::ops::DerefMut;

type E = RecognitionResultEvent;
type C = RecognitionCanceledEvent;

pub struct SpeechRecognizer<CFG, S> {
    base: AbstractAsyncRecognizer<E, C>,
    #[allow(unused)]
    config: CFG,
    #[allow(unused)]
    audio: AudioConfig<S>,
}

impl<CFG, S> SpeechRecognizer<CFG, S>
    where S: Borrow<dyn AudioInputStream>,
          CFG: Borrow<SpeechConfig> {
    pub fn from_config(config: CFG, audio: AudioConfig<S>) -> Result<SpeechRecognizer<CFG, S>, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(
                recognizer_create_speech_recognizer_from_config(
                    &mut handle,
                    config.borrow().get_handle(),
                    audio.get_handle(),
                )
            )?;
        }
        Ok(SpeechRecognizer {
            base: AbstractAsyncRecognizer::create(handle)?,
            config,
            audio,
        })
    }
}

impl<CFG, S> Deref for SpeechRecognizer<CFG, S> {
    type Target = dyn AsyncRecognizer<E, C, Target=dyn Recognizer>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<CFG, S> DerefMut for SpeechRecognizer<CFG, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
