use convert_err;
use futures::future::Future;
use futures::sink::Sink;
use futures::sync::mpsc::{channel, Receiver, Sender};
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
use std::ops::DerefMut;

mod async_handle;
mod results;

const DEFAULT_CH_BUFF_SIZE: usize = 5;

pub trait Recognizer: Send {
    fn is_enabled(&self) -> Result<bool, SpxError>;
    fn enable(&mut self) -> Result<(), SpxError>;
    fn disable(&mut self) -> Result<(), SpxError>;
    fn get_handle(&self) -> SPXRECOHANDLE;
}

pub trait AsyncRecognizer: Deref<Target=dyn Recognizer> {
    fn start_continuous_recognition(&self) -> Result<AsyncHandle, SpxError>;
    fn stop_continuous_recognition(&self) -> Result<AsyncHandle, SpxError>;

    fn set_recognizing_channel(&mut self, v: Option<Sender<usize>>);
    fn set_recognized_channel(&mut self, v: Option<Sender<usize>>);
    fn set_session_started_channel(&mut self, v: Option<Sender<usize>>);
    fn set_session_stopped_channel(&mut self, v: Option<Sender<usize>>);
    fn set_canceled_channel(&mut self, v: Option<Sender<usize>>);

    fn connect_recognizing(&mut self, buff_size: Option<usize>) -> Receiver<usize> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_recognizing_channel(Some(s));
        return r;
    }

    fn connect_recognized(&mut self, buff_size: Option<usize>) -> Receiver<usize> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_recognized_channel(Some(s));
        return r;
    }

    fn connect_session_started(&mut self, buff_size: Option<usize>) -> Receiver<usize> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_session_started_channel(Some(s));
        return r;
    }

    fn connect_session_stopped(&mut self, buff_size: Option<usize>) -> Receiver<usize> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_session_stopped_channel(Some(s));
        return r;
    }

    fn connect_canceled(&mut self, buff_size: Option<usize>) -> Receiver<usize> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_canceled_channel(Some(s));
        return r;
    }
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
    recognizing_sender: Option<Sender<usize>>,
    recognized_sender: Option<Sender<usize>>,
    session_started_sender: Option<Sender<usize>>,
    session_stopped_sender: Option<Sender<usize>>,
    canceled_sender: Option<Sender<usize>>,
}

impl AsyncRecognizer for AbstractAsyncRecognizer {
    fn start_continuous_recognition(&self) -> Result<AsyncHandle, SpxError> {
        self.set_callback(&self.canceled_sender, recognizer_canceled_set_callback);
        self.set_callback(&self.session_started_sender, recognizer_session_started_set_callback);
        self.set_callback(&self.session_stopped_sender, recognizer_session_stopped_set_callback);
        self.set_callback(&self.recognizing_sender, recognizer_recognizing_set_callback);
        self.set_callback(&self.recognized_sender, recognizer_recognized_set_callback);
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

    fn set_recognizing_channel(&mut self, v: Option<Sender<usize>>) {
        self.recognizing_sender = v;
    }

    fn set_recognized_channel(&mut self, v: Option<Sender<usize>>) {
        self.recognized_sender = v;
    }

    fn set_session_started_channel(&mut self, v: Option<Sender<usize>>) {
        self.session_started_sender = v;
    }

    fn set_session_stopped_channel(&mut self, v: Option<Sender<usize>>) {
        self.session_stopped_sender = v;
    }

    fn set_canceled_channel(&mut self, v: Option<Sender<usize>>) {
        self.canceled_sender = v;
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
            recognizing_sender: None,
            recognized_sender: None,
            session_started_sender: None,
            session_stopped_sender: None,
            canceled_sender: None,
        })
    }

    #[inline]
    fn set_callback(&self,
                    sender: &Option<Sender<usize>>,
                    f: unsafe extern "C" fn(SPXRECOHANDLE, PRECOGNITION_CALLBACK_FUNC, *const ::std::os::raw::c_void) -> SPXHR) {
        if let Some(s) = sender {
            unsafe {
                f(self.get_handle(), Some(|_, evt, p_sender| {
                    let sender = unsafe { (*(p_sender as *const Sender<usize>)).clone() };
                    sender.send(1).wait(); // TODO
                }), s as *const _ as *const c_void);
            }
        } else {
            unsafe { f(self.get_handle(), None, 0 as *const c_void); }
        }
    }
}

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