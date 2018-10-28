use convert_err;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use SmartHandle;

#[derive(Debug)]
pub struct AudioStreamFormat {
    handle: SmartHandle<SPXAUDIOSTREAMFORMATHANDLE>,
}

impl AudioStreamFormat {
    pub fn get_default_input_format() -> Result<AudioStreamFormat, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(audio_stream_format_create_from_default_input(&mut handle))?;
        }
        let result = AudioStreamFormat {
            handle: SmartHandle::create(handle, audio_stream_format_release),
        };
        Ok(result)
    }

    pub fn get_wave_format_pcm(samples_per_second: u32, bits_per_sample: Option<u8>, channels: Option<u8>) -> Result<AudioStreamFormat, SpxError> {
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(audio_stream_format_create_from_waveformat_pcm(
                &mut handle,
                samples_per_second,
                bits_per_sample.unwrap_or(16),
                channels.unwrap_or(1),
            ))?;
        }
        let result = AudioStreamFormat {
            handle: SmartHandle::create(handle, audio_stream_format_release),
        };
        Ok(result)
    }

    #[inline]
    pub fn get_handle(&self) -> SPXAUDIOSTREAMFORMATHANDLE {
        self.handle.get()
    }
}
