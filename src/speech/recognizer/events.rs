use CancellationReason;
use convert_err;
use num::FromPrimitive;
use SmartHandle;
use speech::recognizer::RecognitionResult;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ops::Deref;
use std::sync::Arc;

const SESSION_ID_SIZE: usize = 36; // UUID

pub(crate) trait EventFactory<T> {
    fn create(handle: SPXEVENTHANDLE) -> Result<T, SpxError>;
}

// Event

pub struct Event {
    handle: SmartHandle<SPXEVENTHANDLE>,
}

impl EventFactory<Event> for Event {
    #[inline]
    fn create(handle: SPXEVENTHANDLE) -> Result<Event, SpxError> {
        Ok(Event {
            handle: SmartHandle::create(handle, recognizer_event_handle_release),
        })
    }
}

impl Event {
    pub fn get_handle(&self) -> SPXEVENTHANDLE {
        self.handle.get()
    }
}

// SessionEvent

pub struct SessionEvent {
    base: Event,
}

impl Deref for SessionEvent {
    type Target = Event;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl EventFactory<SessionEvent> for SessionEvent {
    #[inline]
    fn create(handle: SPXEVENTHANDLE) -> Result<SessionEvent, SpxError> {
        Ok(SessionEvent {
            base: Event::create(handle)?,
        })
    }
}

impl SessionEvent {
    pub fn session_id(&self) -> Result<String, SpxError> {
        ::spx_populate_string(
            self.get_handle(),
            SESSION_ID_SIZE,
            recognizer_session_event_get_session_id,
        )
    }
}

// RecognitionEvent

pub struct RecognitionEvent {
    base: SessionEvent,
    result_handle: Arc<SmartHandle<SPXRESULTHANDLE>>,
}

impl Deref for RecognitionEvent {
    type Target = SessionEvent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl EventFactory<RecognitionEvent> for RecognitionEvent {
    #[inline]
    fn create(handle: SPXEVENTHANDLE) -> Result<RecognitionEvent, SpxError> {
        Ok(RecognitionEvent {
            base: SessionEvent::create(handle)?,
            result_handle: get_result_handle(handle)?,
        })
    }
}

// RecognitionResultEvent

pub struct RecognitionResultEvent {
    base: RecognitionEvent,
}

impl Deref for RecognitionResultEvent {
    type Target = RecognitionEvent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl EventFactory<RecognitionResultEvent> for RecognitionResultEvent {
    #[inline]
    fn create(handle: SPXEVENTHANDLE) -> Result<RecognitionResultEvent, SpxError> {
        Ok(RecognitionResultEvent {
            base: RecognitionEvent::create(handle)?,
        })
    }
}

impl RecognitionResultEvent {
    pub fn result(&self) -> Result<RecognitionResult, SpxError> {
        RecognitionResult::create(self.result_handle.clone())
    }
}

// RecognitionCanceledEvent

pub struct RecognitionCanceledEvent {
    base: RecognitionEvent,
}


impl Deref for RecognitionCanceledEvent {
    type Target = RecognitionEvent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl EventFactory<RecognitionCanceledEvent> for RecognitionCanceledEvent {
    #[inline]
    fn create(handle: SPXEVENTHANDLE) -> Result<RecognitionCanceledEvent, SpxError> {
        Ok(RecognitionCanceledEvent {
            base: RecognitionEvent::create(handle)?,
        })
    }
}

impl RecognitionCanceledEvent {
    pub fn reason(&self) -> Result<CancellationReason, SpxError> {
        let code = ::spx_populate(self.result_handle.get(), result_get_reason_canceled)?;
        return Ok(CancellationReason::from_u32(code).expect("unknown reason"));
    }
}

#[inline]
fn get_result_handle(event_handle: SPXEVENTHANDLE) -> Result<Arc<SmartHandle<SPXRESULTHANDLE>>, SpxError> {
    let handle = ::spx_populate(event_handle, recognizer_recognition_event_get_result)?;
    Ok(Arc::new(SmartHandle::create(handle, recognizer_result_handle_release)))
}
