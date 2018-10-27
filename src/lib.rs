extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;

use speech_api::*;
use std::ffi;

mod speech_api;
pub mod speech;

const SPXHANDLE_INVALID: SPXHANDLE = 0 as SPXHANDLE;

#[derive(Fail, Debug)]
pub enum SpxError {
    #[fail(display = "Speech API error code: {}.", _0)]
    General(usize),
    #[fail(display = "Invalid CString (NulError).")]
    StrNulError(#[cause] ffi::NulError),
}

impl From<ffi::NulError> for SpxError {
    fn from(err: ffi::NulError) -> Self {
        return SpxError::StrNulError(err);
    }
}

#[inline(always)]
fn convert_err(hr: usize) -> Result<(), SpxError> {
    if hr != SPX_NOERROR {
        return Err(SpxError::General(hr));
    }
    return Ok(());
}

#[derive(Debug)]
pub struct SmartHandle<T: Copy> {
    internal: T,
    release_fn: unsafe extern "C" fn(T) -> SPXHR,
}

impl<T: Copy> SmartHandle<T> {
    fn create(handle: T, release_fn: unsafe extern "C" fn(T) -> SPXHR) -> SmartHandle<T> {
        SmartHandle { internal: handle, release_fn }
    }

    #[inline(always)]
    fn get(&self) -> T {
        self.internal
    }
}

impl<T: Copy> Drop for SmartHandle<T> {
    fn drop(&mut self) {
        unsafe {
            (self.release_fn)(self.internal);
        }
    }
}

unsafe impl<T: Copy> Send for SmartHandle<T> {}
