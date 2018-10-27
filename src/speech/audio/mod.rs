use check_err;
pub use self::stream::AudioInputStream;
pub use self::stream_format::AudioStreamFormat;
use speech_api::*;
use SpxError;
use std::ffi;
use std::sync::Arc;

mod stream;
mod stream_format;

pub struct AudioConfig {
    handle: SPXAUDIOCONFIGHANDLE,
    stream: Arc<dyn AudioInputStream>,
}

impl AudioConfig {
    pub fn from_stream_input(stream: Arc<dyn AudioInputStream>) -> Result<AudioConfig, SpxError> {
        let mut result = AudioConfig {
            handle: 0 as SPXAUDIOCONFIGHANDLE,
            stream,
        };
        unsafe {
            check_err(audio_config_create_audio_input_from_stream(&mut result.handle, result.stream.get_handle()))?;
        }
        Ok(result)
    }
}

impl Drop for AudioConfig {
    fn drop(&mut self) {
        unsafe {
            audio_config_release(self.handle);
        }
    }
}