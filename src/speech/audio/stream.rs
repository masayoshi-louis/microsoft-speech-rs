use convert_err;
use SmartHandle;
use speech::audio::AudioStreamFormat;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

pub trait AudioStreamSink: Send {
    fn write(&self, buf: &[u8]) -> Result<(), SpxError>;

    fn close(&self) -> Result<(), SpxError>;
}

pub trait AudioInputStream: Send {
    fn get_handle(&self) -> SPXAUDIOSTREAMHANDLE;
}

impl AudioInputStream {
    #[inline(always)]
    pub fn create_push_stream(format: Option<AudioStreamFormat>) -> Result<(Arc<dyn AudioInputStream>, Arc<dyn AudioStreamSink>), SpxError> {
        let arc = Arc::new(PushAudioInputStream::create(format)?);
        Ok((arc.clone(), arc))
    }
}

#[derive(Debug)]
struct BaseAudioInputStream {
    handle: SmartHandle<SPXAUDIOSTREAMHANDLE>,
    format: AudioStreamFormat,
}

#[derive(Debug)]
struct PushAudioInputStream {
    base: BaseAudioInputStream,
}

impl PushAudioInputStream {
    fn create(format: Option<AudioStreamFormat>) -> Result<PushAudioInputStream, SpxError> {
        let format = format
            .map(|x| Ok(x))
            .unwrap_or_else(|| { AudioStreamFormat::get_default_input_format() })?;
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(audio_stream_create_push_audio_input_stream(&mut handle, format.get_handle()))?;
        }
        let result = PushAudioInputStream {
            base: BaseAudioInputStream {
                handle: SmartHandle::create(handle, audio_stream_release),
                format,
            }
        };
        Ok(result)
    }
}

impl Drop for PushAudioInputStream {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        unsafe {
            if audio_stream_is_handle_valid(self.handle.get()) {
                self.close();
            }
        }
    }
}

impl AudioInputStream for PushAudioInputStream {
    fn get_handle(&self) -> SPXAUDIOSTREAMHANDLE {
        self.base.handle.get()
    }
}

impl AudioStreamSink for PushAudioInputStream {
    fn write(&self, buf: &[u8]) -> Result<(), SpxError> {
        unsafe {
            convert_err(push_audio_input_stream_write(self.handle.get(), buf.as_ptr(), buf.len() as u32))
        }
    }

    fn close(&self) -> Result<(), SpxError> {
        unsafe {
            convert_err(push_audio_input_stream_close(self.handle.get()))
        }
    }
}

impl Deref for PushAudioInputStream {
    type Target = BaseAudioInputStream;

    fn deref(&self) -> &BaseAudioInputStream {
        &self.base
    }
}

impl DerefMut for PushAudioInputStream {
    fn deref_mut(&mut self) -> &mut BaseAudioInputStream {
        &mut self.base
    }
}
