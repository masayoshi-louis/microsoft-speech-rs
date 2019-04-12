use std;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use futures::prelude::*;
use tokio::timer::Interval;

use crate::convert_err;
use crate::FromHandle;
use crate::SmartHandle;
use crate::speech_api::*;
use crate::SpxError;
use crate::SPXHANDLE_INVALID;

const ACTION_POLL_INTERVAL_MS: u64 = 30;
const RESULT_POLL_INTERVAL_MS: u64 = 100;

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
    handle: Option<SmartHandle<SPXASYNCHANDLE>>,
    timer: Interval,
    async_wait: W,
    // for lazy initialization
    init_handle: SPXHANDLE,
    init_fn: unsafe extern "C" fn(SPXHANDLE, *mut SPXASYNCHANDLE) -> SPXHR,
}

unsafe impl<W> Sync for BaseAsyncHandle<W> {}

unsafe impl<W> Send for BaseAsyncHandle<W> {}

impl<W: AsyncWait> BaseAsyncHandle<W> {
    pub(crate)
    fn create(init_handle: SPXRECOHANDLE,
              init_fn: unsafe extern "C" fn(SPXRECOHANDLE, *mut SPXASYNCHANDLE) -> SPXHR,
              async_wait: W,
              poll_interval: Duration) -> Result<BaseAsyncHandle<W>, SpxError> {
        Ok(BaseAsyncHandle {
            handle: None,
            timer: Interval::new(Instant::now(), poll_interval),
            async_wait,
            init_handle,
            init_fn,
        })
    }
}

impl<W: AsyncWait> Future for BaseAsyncHandle<W> {
    type Item = ();
    type Error = SpxError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if self.handle.is_none() {
            let mut handle = SPXHANDLE_INVALID;
            unsafe {
                convert_err((self.init_fn)(self.init_handle, &mut handle))?;
            }
            self.handle = Some(SmartHandle::create(
                "BaseAsyncHandle",
                handle,
                recognizer_async_handle_release,
            ));
        }
        match self.timer.poll().expect("timer failure") {
            Async::NotReady => Ok(Async::NotReady),
            Async::Ready(_) => {
                let hr = unsafe {
                    self.async_wait.async_wait(self.handle.as_ref().unwrap().get(), 0)
                };
                if hr == SPXERR_TIMEOUT {
                    self.poll()
                } else {
                    convert_err(hr)?;
                    Ok(Async::Ready(()))
                }
            }
        }
    }
}

pub struct AsyncHandle {
    base: BaseAsyncHandle<AsyncWaitFn>,
}

impl AsyncHandle {
    #[inline]
    pub(crate)
    fn create(init_handle: SPXRECOHANDLE,
              init_fn: unsafe extern "C" fn(SPXRECOHANDLE, *mut SPXASYNCHANDLE) -> SPXHR,
              wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32) -> SPXHR) -> Result<AsyncHandle, SpxError> {
        Ok(AsyncHandle {
            base: BaseAsyncHandle::create(
                init_handle,
                init_fn,
                AsyncWaitFn { wait_fn },
                Duration::from_millis(ACTION_POLL_INTERVAL_MS),
            )?
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
    result_handle: Option<Box<SPXRESULTHANDLE>>,
    phantom_v: PhantomData<V>,
}

impl<V> AsyncResultHandle<V> {
    #[inline]
    pub(crate)
    fn create(init_handle: SPXRECOHANDLE,
              init_fn: unsafe extern "C" fn(SPXRECOHANDLE, *mut SPXASYNCHANDLE) -> SPXHR,
              wait_fn: unsafe extern "C" fn(SPXASYNCHANDLE, u32, *mut SPXRESULTHANDLE) -> SPXHR) -> Result<AsyncResultHandle<V>, SpxError> {
        let mut result_handle = Box::new(SPXHANDLE_INVALID);
        let async_wait = AsyncResultWait { wait_fn, result_handle_ptr: &mut *result_handle };
        Ok(AsyncResultHandle {
            base: BaseAsyncHandle::create(init_handle, init_fn, async_wait, Duration::from_millis(RESULT_POLL_INTERVAL_MS))?,
            result_handle: Some(result_handle),
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
                let result_handle =
                    std::mem::replace(&mut self.result_handle, None);
                let smart_handle = Arc::new(SmartHandle::create(
                    "RecognitionResult",
                    *result_handle.expect("result_handle is none"),
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
        if let Some(ref h) = self.result_handle {
            let h = **h;
            if h != SPXHANDLE_INVALID {
                unsafe {
                    recognizer_result_handle_release(h);
                }
            }
        }
    }
}
