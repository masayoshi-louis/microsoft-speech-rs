use convert_err;
pub use self::async_handle::AsyncHandle;
use SmartHandle;
use speech::audio::AudioConfig;
use speech::audio::AudioInputStream;
use speech::SpeechConfig;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ffi::c_void;
use std::ops::Deref;

mod async_handle;

pub trait Recognizer: Send {
    fn is_enabled(&self) -> Result<bool, SpxError>;
    fn enable(&mut self) -> Result<(), SpxError>;
    fn disable(&mut self) -> Result<(), SpxError>;
    fn get_handle(&self) -> SPXRECOHANDLE;
}

pub trait AsyncRecognizer: Deref<Target=dyn Recognizer> {
    fn start_continuous_recognition(&self) -> Result<AsyncHandle, SpxError>;
    fn stop_continuous_recognition(&self) -> Result<AsyncHandle, SpxError>;
}

struct BaseRecognizer {
    handle: SmartHandle<SPXRECOHANDLE>,
}

impl BaseRecognizer {
    fn create(handle: SPXRECOHANDLE) -> Result<BaseRecognizer, SpxError> {
        Ok(BaseRecognizer {
            handle: SmartHandle::create(handle, recognizer_handle_release)
        })
    }
}

impl Recognizer for BaseRecognizer {
    fn is_enabled(&self) -> Result<bool, SpxError> {
        let mut result = false;
        unsafe {
            convert_err(recognizer_is_enabled(self.handle.get(), &mut result))?;
        }
        Ok(result)
    }

    fn enable(&mut self) -> Result<(), SpxError> {
        unsafe {
            convert_err(recognizer_enable(self.handle.get()))
        }
    }

    fn disable(&mut self) -> Result<(), SpxError> {
        unsafe {
            convert_err(recognizer_disable(self.handle.get()))
        }
    }

    fn get_handle(&self) -> SPXRECOHANDLE {
        self.handle.get()
    }
}

struct AbstractAsyncRecognizer {
    base: BaseRecognizer,
}

impl AsyncRecognizer for AbstractAsyncRecognizer {
    fn start_continuous_recognition(&self) -> Result<AsyncHandle, SpxError> {
        AsyncHandle::create(
            self.get_handle(),
            recognizer_start_continuous_recognition_async,
            recognizer_start_continuous_recognition_async_wait_for,
        )
    }

    fn stop_continuous_recognition(&self) -> Result<AsyncHandle, SpxError> {
        AsyncHandle::create(
            self.get_handle(),
            recognizer_stop_continuous_recognition_async,
            recognizer_stop_continuous_recognition_async_wait_for,
        )
    }
}

impl Deref for AbstractAsyncRecognizer {
    type Target = dyn Recognizer;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl AbstractAsyncRecognizer {
    fn create(handle: SPXRECOHANDLE) -> Result<AbstractAsyncRecognizer, SpxError> {
        Ok(AbstractAsyncRecognizer {
            base: BaseRecognizer::create(handle)?,
        })
    }
}

pub struct SpeechRecognizer {
    base: AbstractAsyncRecognizer,
}

impl SpeechRecognizer {
    pub fn from_config<S>(config: SpeechConfig, audio: AudioConfig<S>) -> Result<SpeechRecognizer, SpxError>
        where S: AsRef<dyn AudioInputStream> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(recognizer_create_speech_recognizer_from_config(&mut handle, config.get_handle(), audio.get_handle()))?;
            recognizer_recognizing_set_callback(handle, Some(|h, e, c| {
                println!("test recognizing");
            }), 0 as *mut c_void);
        }
        Ok(SpeechRecognizer {
            base: AbstractAsyncRecognizer::create(handle)?,
        })
    }
}

impl Deref for SpeechRecognizer {
    type Target = dyn AsyncRecognizer<Target=dyn Recognizer>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
