use convert_err;
use SmartHandle;
use speech_api::*;
use SpxError;

mod async_handle;

pub trait Recognizer: Send {
    fn is_enabled(&self) -> Result<bool, SpxError>;

    fn enable(&mut self) -> Result<(), SpxError>;

    fn disable(&mut self) -> Result<(), SpxError>;
}

pub trait AsyncRecognizer: Recognizer {}

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
}

