use convert_err;
pub use self::stream::AudioInputStream;
pub use self::stream::AudioStreamSink;
pub use self::stream_format::AudioStreamFormat;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ffi;
use std::sync::Arc;

mod stream;
mod stream_format;

pub struct AudioConfig<S> {
    handle: SPXAUDIOCONFIGHANDLE,
    stream: S,
}

impl<S: AsRef<dyn AudioInputStream>> AudioConfig<S> {
    pub fn from_stream_input(stream: S) -> Result<AudioConfig<S>, SpxError> {
        let mut result = AudioConfig {
            handle: SPXHANDLE_INVALID,
            stream,
        };
        unsafe {
            convert_err(audio_config_create_audio_input_from_stream(
                &mut result.handle,
                result.stream.as_ref().get_handle(),
            ))?;
        }
        Ok(result)
    }
}

impl<S> Drop for AudioConfig<S> {
    fn drop(&mut self) {
        unsafe {
            audio_config_release(self.handle);
        }
    }
}