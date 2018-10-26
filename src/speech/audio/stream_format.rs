use check_err;
use speech_api::*;
use SpxError;
use std::ffi;

#[derive(Debug)]
pub struct AudioStreamFormat {
    handle: SPXAUDIOSTREAMFORMATHANDLE,
}

impl AudioStreamFormat {
    pub fn get_default_input_format() -> Result<AudioStreamFormat, SpxError> {
        let mut result = AudioStreamFormat {
            handle: 0 as SPXAUDIOSTREAMFORMATHANDLE,
        };
        unsafe {
            check_err(audio_stream_format_create_from_default_input(&mut result.handle))?;
        }
        Ok(result)
    }

    pub fn get_handle(&self) -> SPXAUDIOSTREAMFORMATHANDLE {
        return self.handle;
    }
}

impl Drop for AudioStreamFormat {
    fn drop(&mut self) {
        unsafe {
            audio_stream_format_release(self.handle);
        }
    }
}
