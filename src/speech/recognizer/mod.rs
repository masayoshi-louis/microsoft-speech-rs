use convert_err;
use futures::future::Future;
use futures::sink::Sink;
use futures::sync::mpsc::{channel, Receiver, Sender};
use num::FromPrimitive;
use ResultReason;
pub use self::async_handle::AsyncHandle;
pub use self::speech::*;
use SmartHandle;
use speech::recognizer::events::EventFactory;
use speech::recognizer::events::SessionEvent;
use speech_api::*;
use SpxError;
use std::ffi::c_void;
use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::Arc;
use std::time::Duration;

mod async_handle;
pub mod events;
mod speech;

const DEFAULT_CH_BUFF_SIZE: usize = 5;
const MAX_RESULT_ID_CHAR_COUNT: usize = 128;
const MAX_TEXT_CHAR_COUNT: usize = 1024;

pub trait Recognizer: Send {
    fn is_enabled(&self) -> Result<bool, SpxError>;
    fn enable(&mut self) -> Result<(), SpxError>;
    fn disable(&mut self) -> Result<(), SpxError>;
    fn get_handle(&self) -> SPXRECOHANDLE;
}

pub trait AsyncRecognizer<E, C>: Deref<Target=dyn Recognizer>
    where E: EventFactory<R=E>, C: EventFactory<R=C> {
    fn start_continuous_recognition(&self) -> Result<AsyncHandle, SpxError>;
    fn stop_continuous_recognition(&self) -> Result<AsyncHandle, SpxError>;

    fn set_recognizing_channel(&mut self, v: Option<Box<Sender<E>>>);
    fn set_recognized_channel(&mut self, v: Option<Box<Sender<E>>>);
    fn set_session_started_channel(&mut self, v: Option<Box<Sender<SessionEvent>>>);
    fn set_session_stopped_channel(&mut self, v: Option<Box<Sender<SessionEvent>>>);
    fn set_canceled_channel(&mut self, v: Option<Box<Sender<C>>>);

    fn connect_recognizing(&mut self, buff_size: Option<usize>) -> Receiver<E> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_recognizing_channel(Some(Box::new(s)));
        return r;
    }

    fn connect_recognized(&mut self, buff_size: Option<usize>) -> Receiver<E> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_recognized_channel(Some(Box::new(s)));
        return r;
    }

    fn connect_session_started(&mut self, buff_size: Option<usize>) -> Receiver<SessionEvent> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_session_started_channel(Some(Box::new(s)));
        return r;
    }

    fn connect_session_stopped(&mut self, buff_size: Option<usize>) -> Receiver<SessionEvent> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_session_stopped_channel(Some(Box::new(s)));
        return r;
    }

    fn connect_canceled(&mut self, buff_size: Option<usize>) -> Receiver<C> {
        let (s, r) = channel(buff_size.unwrap_or(DEFAULT_CH_BUFF_SIZE));
        self.set_canceled_channel(Some(Box::new(s)));
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

struct AbstractAsyncRecognizer<E, C> {
    base: BaseRecognizer,
    recognizing_sender: Option<Box<Sender<E>>>,
    recognized_sender: Option<Box<Sender<E>>>,
    session_started_sender: Option<Box<Sender<SessionEvent>>>,
    session_stopped_sender: Option<Box<Sender<SessionEvent>>>,
    canceled_sender: Option<Box<Sender<C>>>,
}

impl<E, C> AsyncRecognizer<E, C> for AbstractAsyncRecognizer<E, C>
    where E: EventFactory<R=E>, C: EventFactory<R=C> {
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

    fn set_recognizing_channel(&mut self, v: Option<Box<Sender<E>>>) {
        self.recognizing_sender = v;
    }

    fn set_recognized_channel(&mut self, v: Option<Box<Sender<E>>>) {
        self.recognized_sender = v;
    }

    fn set_session_started_channel(&mut self, v: Option<Box<Sender<SessionEvent>>>) {
        self.session_started_sender = v;
    }

    fn set_session_stopped_channel(&mut self, v: Option<Box<Sender<SessionEvent>>>) {
        self.session_stopped_sender = v;
    }

    fn set_canceled_channel(&mut self, v: Option<Box<Sender<C>>>) {
        self.canceled_sender = v;
    }
}

impl<E, C> Deref for AbstractAsyncRecognizer<E, C> {
    type Target = dyn Recognizer;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<E, C> AbstractAsyncRecognizer<E, C> {
    fn create(handle: SPXRECOHANDLE) -> Result<AbstractAsyncRecognizer<E, C>, SpxError> {
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
    fn set_callback<T>(&self,
                       sender: &Option<Box<Sender<T>>>,
                       f: unsafe extern "C" fn(SPXRECOHANDLE, PRECOGNITION_CALLBACK_FUNC, *const ::std::os::raw::c_void) -> SPXHR)
        where T: EventFactory<R=T> {
        if let Some(s) = sender {
            let s = s.as_ref();
            let cb: PRECOGNITION_CALLBACK_FUNC = Some(|_, h_evt, p_sender| {
                let sender = unsafe { &mut *(p_sender as *mut Sender<T>) };
                let event = match T::create(h_evt) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("can not create event, err: {}", e);
                        return;
                    }
                };
                match sender.try_send(event) {
                    Ok(()) => {}
                    Err(e) => {
                        error!("can not publish event, err: {}", e);
                    }
                }
            });
            unsafe {
                f(self.get_handle(), cb, s as *const _ as *const c_void);
            }
        } else {
            unsafe {
                f(self.get_handle(), None, 0 as *const c_void);
            }
        }
    }
}

pub struct RecognitionResult {
    handle: Arc<SmartHandle<SPXRESULTHANDLE>>,
}

impl RecognitionResult {
    fn create(handle: Arc<SmartHandle<SPXRESULTHANDLE>>) -> Result<RecognitionResult, SpxError> {
        Ok(RecognitionResult {
            handle,
        })
    }

    #[inline(always)]
    pub fn get_handle(&self) -> SPXRESULTHANDLE {
        self.handle.get()
    }

    pub fn id(&self) -> Result<String, SpxError> {
        self.populate_string(MAX_RESULT_ID_CHAR_COUNT, result_get_result_id)
    }

    pub fn text(&self) -> Result<String, SpxError> {
        self.populate_string(MAX_TEXT_CHAR_COUNT, result_get_text)
    }

    pub fn reason(&self) -> Result<ResultReason, SpxError> {
        let code = ::spx_populate(self.get_handle(), result_get_reason)?;
        return Ok(ResultReason::from_u32(code).expect("unknown reason"));
    }

    pub fn offset(&self) -> Result<u64, SpxError> {
        self.populate(result_get_offset)
    }

    pub fn duration(&self) -> Result<Duration, SpxError> {
        self.populate(result_get_offset).map(Duration::from_millis)
    }

    #[inline(always)]
    fn populate_string(&self, max_chars: usize,
                       f: unsafe extern "C" fn(SPXRESULTHANDLE, *mut c_char, u32) -> SPXHR) -> Result<String, SpxError> {
        ::spx_populate_string(self.get_handle(), max_chars, f)
    }

    #[inline(always)]
    fn populate<T>(&self,
                   f: unsafe extern "C" fn(SPXRESULTHANDLE, *mut T) -> SPXHR) -> Result<T, SpxError> {
        ::spx_populate(self.get_handle(), f)
    }
}
