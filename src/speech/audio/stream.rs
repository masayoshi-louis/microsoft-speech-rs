use convert_err;
use speech::audio::AudioStreamFormat;
use speech_api::*;
use SpxError;
use std::ops::Deref;

pub trait AudioInputStream {
    fn get_handle(&self) -> SPXAUDIOSTREAMHANDLE;
}

impl AudioInputStream {
    #[inline(always)]
    pub fn create_push_stream(format: Option<AudioStreamFormat>) -> Result<impl AudioInputStream, SpxError> {
        return PushAudioInputStream::create(format);
    }
}

#[derive(Debug)]
struct BaseAudioInputStream {
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
struct PushAudioInputStream {
    base: BaseAudioInputStream,
}

impl PushAudioInputStream {
    fn create(format: Option<AudioStreamFormat>) -> Result<PushAudioInputStream, SpxError> {
        let format = format
            .map(|x| Ok(x))
            .unwrap_or_else(|| { AudioStreamFormat::get_default_input_format() })?;
        let mut result = PushAudioInputStream {
            base: BaseAudioInputStream {
                handle: 0 as SPXAUDIOSTREAMHANDLE,
                format,
            }
        };
        unsafe {
            convert_err(audio_stream_create_push_audio_input_stream(&mut result.handle, result.format.get_handle()))?;
        }
        Ok(result)
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
