use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use futures::prelude::*;
use tokio::timer::Interval;

use convert_err;
use FromHandle;
use SmartHandle;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;

const PULL_INTERVAL_MS: u64 = 30;

const SPXERR_TIMEOUT: SPXHR = 0x06;

pub struct AsyncHandle {
    handle: SmartHandle<SPXASYNCHANDLE>,
    wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32) -> SPXHR,
    timer: Interval,
}

impl Deref for AsyncHandle {
    type Target = SmartHandle<SPXASYNCHANDLE>;

    fn deref(&self) -> &SmartHandle<SPXASYNCHANDLE> {
        &self.handle
    }
}

impl AsyncHandle {
    pub(crate)
    fn create(hreco: SPXRECOHANDLE,
              init_fn: unsafe extern "C" fn(SPXRECOHANDLE, *mut SPXASYNCHANDLE) -> SPXHR,
              wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32) -> SPXHR) -> Result<AsyncHandle, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(init_fn(hreco, &mut handle))?;
        }
        Ok(AsyncHandle {
            handle: SmartHandle::create("AsyncHandle", handle, recognizer_async_handle_release),
            wait_fn,
            timer: Interval::new(Instant::now(), Duration::from_millis(PULL_INTERVAL_MS)),
        })
    }
}

impl Future for AsyncHandle {
    type Item = ();
    type Error = SpxError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let hr = unsafe {
            (self.wait_fn)(self.handle.get(), 0)
        };
        if hr == SPXERR_TIMEOUT {
            match self.timer.poll().expect("timer failure") {
                Async::NotReady => return Ok(Async::NotReady),
                Async::Ready(_) => return self.poll(),
            }
        }
        convert_err(hr)?;
        Ok(Async::Ready(()))
    }
}

pub struct AsyncResultHandle<V> {
    handle: SmartHandle<SPXASYNCHANDLE>,
    timer: Interval,
    phantom_v: PhantomData<V>,
}

impl<V> AsyncResultHandle<V> {
    pub(crate)
    fn create(hreco: SPXRECOHANDLE) -> Result<AsyncResultHandle<V>, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(recognizer_recognize_once_async(hreco, &mut handle))?;
        }
        Ok(AsyncResultHandle {
            handle: SmartHandle::create("AsyncResultHandle", handle, recognizer_async_handle_release),
            timer: Interval::new(Instant::now(), Duration::from_millis(PULL_INTERVAL_MS)),
            phantom_v: PhantomData,
        })
    }
}

impl<V> Future for AsyncResultHandle<V>
    where V: FromHandle<Handle=Arc<SmartHandle<SPXRESULTHANDLE>>, Err=SpxError> {
    type Item = V;
    type Error = SpxError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut result_handle = SPXHANDLE_INVALID;
        let hr = unsafe {
            recognizer_recognize_once_async_wait_for(self.handle.get(), 0, &mut result_handle)
        };
        if hr == SPXERR_TIMEOUT {
            match self.timer.poll().expect("timer failure") {
                Async::NotReady => return Ok(Async::NotReady),
                Async::Ready(_) => return self.poll(),
            }
        }
        convert_err(hr)?;
        let result = V::from_handle(Arc::new(SmartHandle::create(
            "RecognitionResult",
            result_handle,
            recognizer_result_handle_release,
        )))?;
        Ok(Async::Ready(result))
    }
}
