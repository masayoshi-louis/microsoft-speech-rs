use convert_err;
use speech::audio::AudioConfig;
use speech::audio::AudioInputStream;
use speech::recognizer::AbstractAsyncRecognizer;
use speech::recognizer::AsyncRecognizer;
use speech::recognizer::Recognizer;
use speech::SpeechConfig;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ops::Deref;
use std::ops::DerefMut;

pub struct SpeechRecognizer<S> {
    base: AbstractAsyncRecognizer,
    config: SpeechConfig,
    audio: AudioConfig<S>,
}

impl<S: AsRef<dyn AudioInputStream>> SpeechRecognizer<S> {
    pub fn from_config(config: SpeechConfig, audio: AudioConfig<S>) -> Result<SpeechRecognizer<S>, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(recognizer_create_speech_recognizer_from_config(&mut handle, config.get_handle(), audio.get_handle()))?;
        }
        Ok(SpeechRecognizer {
            base: AbstractAsyncRecognizer::create(handle)?,
            config,
            audio,
        })
    }
}

impl<S> Deref for SpeechRecognizer<S> {
    type Target = dyn AsyncRecognizer<Target=dyn Recognizer>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<S> DerefMut for SpeechRecognizer<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
