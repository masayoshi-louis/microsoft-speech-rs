use convert_err;
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
            convert_err(audio_stream_format_create_from_default_input(&mut result.handle))?;
        }
        Ok(result)
    }

    pub fn get_wave_format_pcm(samples_per_second: u32, bits_per_sample: Option<u8>, channels: Option<u8>) -> Result<AudioStreamFormat, SpxError> {
        let mut result = AudioStreamFormat {
            handle: 0 as SPXAUDIOSTREAMFORMATHANDLE,
        };
        unsafe {
            convert_err(audio_stream_format_create_from_waveformat_pcm(
                &mut result.handle,
                samples_per_second,
                bits_per_sample.unwrap_or(16),
                channels.unwrap_or(1),
            ))?;
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
