#[macro_use]
extern crate enum_primitive;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
extern crate num;

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
    #[fail(display = "Failed to convert C string.")]
    IntoStringError(#[cause] ffi::IntoStringError),
}

impl From<ffi::NulError> for SpxError {
    fn from(err: ffi::NulError) -> Self {
        return SpxError::StrNulError(err);
    }
}

impl From<ffi::IntoStringError> for SpxError {
    fn from(err: ffi::IntoStringError) -> Self {
        return SpxError::IntoStringError(err);
    }
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum ResultReason {
    /// <summary>
    /// Indicates speech could not be recognized. More details can be found in the NoMatchDetails object.
    /// </summary>
    NoMatch = 0,

    /// <summary>
    /// Indicates that the recognition was canceled. More details can be found using the CancellationDetails object.
    /// </summary>
    Canceled = 1,

    /// <summary>
    /// Indicates the speech result contains hypothesis text.
    /// </summary>
    RecognizingSpeech = 2,

    /// <summary>
    /// Indicates the speech result contains final text that has been recognized.
    /// Speech Recognition is now complete for this phrase.
    /// </summary>
    RecognizedSpeech = 3,

    /// <summary>
    /// Indicates the intent result contains hypothesis text and intent.
    /// </summary>
    RecognizingIntent = 4,

    /// <summary>
    /// Indicates the intent result contains final text and intent.
    /// Speech Recognition and Intent determination are now complete for this phrase.
    /// </summary>
    RecognizedIntent = 5,

    /// <summary>
    /// Indicates the translation result contains hypothesis text and its translation(s).
    /// </summary>
    TranslatingSpeech = 6,

    /// <summary>
    /// Indicates the translation result contains final text and corresponding translation(s).
    /// Speech Recognition and Translation are now complete for this phrase.
    /// </summary>
    TranslatedSpeech = 7,

    /// <summary>
    /// Indicates the synthesized audio result contains a non-zero amount of audio data
    /// </summary>
    SynthesizingAudio = 8,

    /// <summary>
    /// Indicates the synthesized audio is now complete for this phrase.
    /// </summary>
    SynthesizingAudioCompleted = 9,
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
