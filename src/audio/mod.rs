use convert_err;
pub use self::stream::AudioInputStream;
pub use self::stream::AudioStreamSink;
pub use self::stream::PullAudioInputStreamCallback;
pub use self::stream_format::AudioStreamFormat;
use SmartHandle;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::borrow::Borrow;
use std::ffi::CString;

mod stream;
mod stream_format;

pub struct AudioConfig<S> {
    handle: SmartHandle<SPXAUDIOCONFIGHANDLE>,
    #[allow(unused)]
    stream: Option<S>,
}

impl<S: Borrow<dyn AudioInputStream>> AudioConfig<S> {
    pub fn from_stream_input(stream: S) -> Result<AudioConfig<S>, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(audio_config_create_audio_input_from_stream(
                &mut handle,
                stream.borrow().get_handle(),
            ))?;
        }
        let result = AudioConfig {
            handle: SmartHandle::create("AudioConfig", handle, audio_config_release),
            stream: Some(stream),
        };
        Ok(result)
    }

    pub fn from_wav_file_input<NM: AsRef<str>>(file_name: NM) -> Result<AudioConfig<S>, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        let c_file_name = CString::new(file_name.as_ref())?;
        unsafe {
            convert_err(audio_config_create_audio_input_from_wav_file_name(
                &mut handle,
                c_file_name.as_ptr(),
            ))?;
        }
        let result = AudioConfig {
            handle: SmartHandle::create("AudioConfig", handle, audio_config_release),
            stream: None,
        };
        Ok(result)
    }

    #[inline]
    pub fn get_handle(&self) -> SPXAUDIOCONFIGHANDLE {
        self.handle.get()
    }
}
