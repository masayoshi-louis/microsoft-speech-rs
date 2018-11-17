use std;
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

pub trait AsyncWait {
    unsafe fn async_wait(&self, hasync: SPXASYNCHANDLE, timeout: u32) -> SPXHR;
}

pub struct AsyncWaitFn {
    wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32) -> SPXHR,
}

impl AsyncWait for AsyncWaitFn {
    unsafe fn async_wait(&self, hasync: SPXASYNCHANDLE, timeout: u32) -> SPXHR {
        (self.wait_fn)(hasync, timeout)
    }
}

pub struct BaseAsyncHandle<W> {
    handle: SmartHandle<SPXASYNCHANDLE>,
    timer: Interval,
    async_wait: W,
}

impl<W> Deref for BaseAsyncHandle<W> {
    type Target = SmartHandle<SPXASYNCHANDLE>;

    fn deref(&self) -> &SmartHandle<SPXASYNCHANDLE> {
        &self.handle
    }
}

impl<W: AsyncWait> BaseAsyncHandle<W> {
    pub(crate)
    fn create(hreco: SPXRECOHANDLE,
              init_fn: unsafe extern "C" fn(SPXRECOHANDLE, *mut SPXASYNCHANDLE) -> SPXHR,
              async_wait: W) -> Result<BaseAsyncHandle<W>, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(init_fn(hreco, &mut handle))?;
        }
        Ok(BaseAsyncHandle {
            handle: SmartHandle::create("BaseAsyncHandle", handle, recognizer_async_handle_release),
            timer: Interval::new(Instant::now(), Duration::from_millis(PULL_INTERVAL_MS)),
            async_wait,
        })
    }
}

impl<W: AsyncWait> Future for BaseAsyncHandle<W> {
    type Item = ();
    type Error = SpxError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let hr = unsafe {
            self.async_wait.async_wait(self.handle.get(), 0)
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

pub struct AsyncHandle {
    base: BaseAsyncHandle<AsyncWaitFn>,
}

impl AsyncHandle {
    pub(crate)
    fn create(hreco: SPXRECOHANDLE,
              init_fn: unsafe extern "C" fn(SPXRECOHANDLE, *mut SPXASYNCHANDLE) -> SPXHR,
              wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32) -> SPXHR) -> Result<AsyncHandle, SpxError> {
        Ok(AsyncHandle {
            base: BaseAsyncHandle::create(hreco, init_fn, AsyncWaitFn { wait_fn })?
        })
    }
}

impl Future for AsyncHandle {
    type Item = ();
    type Error = SpxError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.base.poll()
    }
}

pub struct AsyncResultWait {
    wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32, *mut SPXRESULTHANDLE) -> SPXHR,
    result_handle_ptr: *mut SPXRESULTHANDLE,
}

impl AsyncWait for AsyncResultWait {
    unsafe fn async_wait(&self, hasync: SPXASYNCHANDLE, timeout: u32) -> SPXHR {
        (self.wait_fn)(hasync, timeout, self.result_handle_ptr)
    }
}

pub struct AsyncResultHandle<V> {
    base: BaseAsyncHandle<AsyncResultWait>,
    result_handle: Box<SPXRESULTHANDLE>,
    phantom_v: PhantomData<V>,
}

impl<V> AsyncResultHandle<V> {
    pub(crate)
    fn create(hreco: SPXRECOHANDLE,
              init_fn: unsafe extern "C" fn(SPXRECOHANDLE, *mut SPXASYNCHANDLE) -> SPXHR,
              wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32, *mut SPXRESULTHANDLE) -> SPXHR) -> Result<AsyncResultHandle<V>, SpxError> {
        let mut result_handle = Box::new(SPXHANDLE_INVALID);
        let async_wait = AsyncResultWait { wait_fn, result_handle_ptr: &mut *result_handle };
        Ok(AsyncResultHandle {
            base: BaseAsyncHandle::create(hreco, init_fn, async_wait)?,
            result_handle,
            phantom_v: PhantomData,
        })
    }
}

impl<V> Future for AsyncResultHandle<V>
    where V: FromHandle<Handle=Arc<SmartHandle<SPXRESULTHANDLE>>, Err=SpxError> {
    type Item = V;
    type Error = SpxError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.base.poll()? {
            Async::NotReady => Ok(Async::NotReady),
            Async::Ready(_) => {
                let mut result_handle = Box::new(SPXHANDLE_INVALID);
                std::mem::swap(&mut self.result_handle, &mut result_handle);
                let smart_handle = Arc::new(SmartHandle::create(
                    "RecognitionResult",
                    *result_handle,
                    recognizer_result_handle_release,
                ));
                let v = V::from_handle(smart_handle)?;
                Ok(Async::Ready(v))
            }
        }
    }
}

impl<V> Drop for AsyncResultHandle<V> {
    fn drop(&mut self) {
        if *self.result_handle != SPXHANDLE_INVALID {
            unsafe {
                recognizer_result_handle_release(*self.result_handle);
            }
        }
    }
}
