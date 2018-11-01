use audio::AudioStreamFormat;
use convert_err;
use SmartHandle;
use speech_api::*;
use SpxError;
use SPXHANDLE_INVALID;
use std::ffi::c_void;
use std::ops::Deref;
use std::ops::DerefMut;
use std::slice;
use std::sync::Arc;

pub trait AudioStreamSink: Send {
    fn write(&self, buf: &[u8]) -> Result<(), SpxError>;

    fn close(&self) -> Result<(), SpxError>;
}

pub trait AudioInputStream: Send {
    fn get_handle(&self) -> SPXAUDIOSTREAMHANDLE;
}

impl AudioInputStream {
    pub fn create_push_stream(format: Option<AudioStreamFormat>) -> Result<(Arc<dyn AudioInputStream>, Arc<dyn AudioStreamSink>), SpxError> {
        let arc = Arc::new(PushAudioInputStream::create(format)?);
        Ok((arc.clone(), arc))
    }

    pub fn create_pull_stream<CB>(callback: CB, format: Option<AudioStreamFormat>) -> Result<Box<dyn AudioInputStream>, SpxError>
        where CB: PullAudioInputStreamCallback + 'static {
        Ok(Box::new(PullAudioInputStream::create(format, callback)?))
    }
}

#[derive(Debug)]
struct BaseAudioInputStream {
    handle: SmartHandle<SPXAUDIOSTREAMHANDLE>,
    format: AudioStreamFormat,
}

impl BaseAudioInputStream {
    fn create(name: &'static str,
              format: Option<AudioStreamFormat>,
              create_fn: unsafe extern "C" fn(*mut SPXAUDIOSTREAMHANDLE, SPXAUDIOSTREAMFORMATHANDLE) -> SPXHR) -> Result<BaseAudioInputStream, SpxError> {
        let format = format
            .map(|x| Ok(x))
            .unwrap_or_else(|| { AudioStreamFormat::get_default_input_format() })?;
        let mut handle = SPXHANDLE_INVALID;
        unsafe {
            convert_err(create_fn(&mut handle, format.get_handle()))?;
        }
        let result = BaseAudioInputStream {
            handle: SmartHandle::create(name, handle, audio_stream_release),
            format,
        };
        Ok(result)
    }
}

// PushAudioInputStream

#[derive(Debug)]
struct PushAudioInputStream {
    base: BaseAudioInputStream,
}

impl PushAudioInputStream {
    fn create(format: Option<AudioStreamFormat>) -> Result<PushAudioInputStream, SpxError> {
        Ok(PushAudioInputStream {
            base: BaseAudioInputStream::create("PushAudioInputStream", format, audio_stream_create_push_audio_input_stream)?,
        })
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
    #[inline]
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

// PullAudioInputStream

pub trait PullAudioInputStreamCallback: Send {
    fn read(&mut self, data_buffer: &mut [u8]) -> usize;
    fn close(&mut self);
}

struct PullAudioInputStream<CB> {
    base: BaseAudioInputStream,
    callback: Box<CB>,
}

impl<CB> PullAudioInputStream<CB> where CB: PullAudioInputStreamCallback + 'static {
    fn create(format: Option<AudioStreamFormat>, callback: CB) -> Result<PullAudioInputStream<CB>, SpxError> {
        let mut result = PullAudioInputStream {
            base: BaseAudioInputStream::create("PullAudioInputStream", format, audio_stream_create_pull_audio_input_stream)?,
            callback: Box::new(callback),
        };

        let read_cb: CUSTOM_AUDIO_PULL_STREAM_READ_CALLBACK = Some(|pv_ctx, buff, size| {
            let cb = unsafe { &mut *(pv_ctx as *mut CB) };
            let buff = unsafe { slice::from_raw_parts_mut(buff, size as usize) };
            cb.read(buff) as i32
        });
        let close_cb: CUSTOM_AUDIO_PULL_STREAM_CLOSE_CALLBACK = Some(|pv_ctx| {
            let cb = unsafe { &mut *(pv_ctx as *mut CB) };
            cb.close();
        });

        unsafe {
            let cb_ptr = &mut *result.callback as *mut _ as *mut c_void;
            convert_err(pull_audio_input_stream_set_callbacks(result.get_handle(), cb_ptr, read_cb, close_cb))?;
        }

        Ok(result)
    }
}

impl<CB> Deref for PullAudioInputStream<CB> {
    type Target = BaseAudioInputStream;

    fn deref(&self) -> &BaseAudioInputStream {
        &self.base
    }
}

impl<CB: Send> AudioInputStream for PullAudioInputStream<CB> {
    #[inline(always)]
    fn get_handle(&self) -> SPXAUDIOSTREAMHANDLE {
        self.base.handle.get()
    }
}
