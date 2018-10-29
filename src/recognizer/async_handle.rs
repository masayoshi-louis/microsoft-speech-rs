use convert_err;
use futures::prelude::*;
use SmartHandle;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ops::Deref;
use std::time::Duration;
use std::time::Instant;
use tokio::timer::Interval;

const PULL_INTERVAL_MS: u64 = 30;

pub struct AsyncHandle {
    handle: SmartHandle<SPXASYNCHANDLE>,
    wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32) -> SPXHR,
    timer: Interval,
}

const SPXERR_TIMEOUT: SPXHR = 0x06;

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
