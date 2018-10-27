use futures::prelude::*;
use SmartHandle;
use speech_api::*;
use SpxError;
use std::ops::Deref;

pub struct AsyncHandle(SmartHandle<SPXASYNCHANDLE>);

impl Deref for AsyncHandle {
    type Target = SmartHandle<SPXASYNCHANDLE>;

    fn deref(&self) -> &SmartHandle<SPXASYNCHANDLE> {
        &self.0
    }
}
