use convert_err;
use speech::audio::AudioStreamFormat;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ops::Deref;
use std::ops::DerefMut;

pub trait AudioInputStream {
    fn get_handle(&self) -> SPXAUDIOSTREAMHANDLE;
}

impl AudioInputStream {
    #[inline(always)]
    pub fn create_push_stream(format: Option<AudioStreamFormat>) -> Result<PushAudioInputStream, SpxError> {
        return PushAudioInputStream::create(format);
    }
}

#[derive(Debug)]
pub struct BaseAudioInputStream {
    handle: SPXAUDIOSTREAMHANDLE,
    format: AudioStreamFormat,
}

impl Drop for BaseAudioInputStream {
    fn drop(&mut self) {
        unsafe {
            audio_stream_release(self.handle);
        }
    }
}

#[derive(Debug)]
pub struct PushAudioInputStream {
    base: BaseAudioInputStream,
}

impl PushAudioInputStream {
    fn create(format: Option<AudioStreamFormat>) -> Result<PushAudioInputStream, SpxError> {
        let format = format
            .map(|x| Ok(x))
            .unwrap_or_else(|| { AudioStreamFormat::get_default_input_format() })?;
        let mut result = PushAudioInputStream {
            base: BaseAudioInputStream {
                handle: SPXHANDLE_INVALID,
                format,
            }
        };
        unsafe {
            convert_err(audio_stream_create_push_audio_input_stream(&mut result.handle, result.format.get_handle()))?;
        }
        Ok(result)
    }

    pub fn write<T: AsRef<[u8]>>(&self, data_buffer: T, size: u32) -> Result<(), SpxError> {
        unsafe {
            convert_err(push_audio_input_stream_write(self.handle, data_buffer.as_ref().as_ptr(), size))
        }
    }

    pub fn close(&self) -> Result<(), SpxError> {
        unsafe {
            convert_err(push_audio_input_stream_close(self.handle))
        }
    }
}

impl Drop for PushAudioInputStream {
    fn drop(&mut self) {
        unsafe {
            if audio_stream_is_handle_valid(self.handle) {
                self.close();
            }
        }
    }
}

impl AudioInputStream for PushAudioInputStream {
    fn get_handle(&self) -> SPXAUDIOSTREAMHANDLE {
        self.base.handle
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
