use convert_err;
use futures::prelude::*;
use futures::task;
use SmartHandle;
use speech_api::*;
use SpxError;
use std::ops::Deref;
use std::time::Duration;
use SPXHANDLE_INVALID;

pub struct AsyncHandle {
    handle: SmartHandle<SPXASYNCHANDLE>,
    wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32) -> SPXHR,
}

//const SPXERR_TIMEOUT: SPXHR = 0x06;

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
            handle: SmartHandle::create(handle, recognizer_async_handle_release),
            wait_fn,
        })
    }
}

impl Future for AsyncHandle {
    type Item = ();
    type Error = SpxError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let hr = unsafe {
            (self.wait_fn)(self.handle.get(), 120_000)
        };
        convert_err(hr)?;
        Ok(Async::Ready(()))
    }
}
