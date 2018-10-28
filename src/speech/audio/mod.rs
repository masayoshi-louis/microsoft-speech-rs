use convert_err;
pub use self::stream::AudioInputStream;
pub use self::stream::AudioStreamSink;
pub use self::stream_format::AudioStreamFormat;
use SmartHandle;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;

mod stream;
mod stream_format;

pub struct AudioConfig<S> {
    handle: SmartHandle<SPXAUDIOCONFIGHANDLE>,
    #[allow(unused)]
    stream: S,
}

impl<S: AsRef<dyn AudioInputStream>> AudioConfig<S> {
    pub fn from_stream_input(stream: S) -> Result<AudioConfig<S>, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(audio_config_create_audio_input_from_stream(
                &mut handle,
                stream.as_ref().get_handle(),
            ))?;
        }
        let result = AudioConfig {
            handle: SmartHandle::create(handle, audio_config_release),
            stream,
        };
        Ok(result)
    }

    pub fn get_handle(&self) -> SPXAUDIOCONFIGHANDLE {
        self.handle.get()
    }
}
