extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate microsoft_speech;
extern crate tokio;

use futures::stream::MergedItem;
use microsoft_speech::audio::AudioConfig;
use microsoft_speech::audio::AudioInputStream;
use microsoft_speech::PropertyId;
use microsoft_speech::recognizer::events::RecognitionResultEvent;
use microsoft_speech::recognizer::SpeechRecognizer;
use microsoft_speech::SpeechConfig;
use std::env;
use std::fs::File;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;
use tokio::prelude::*;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    info!("start");

    let (audio_stream, sink) = AudioInputStream::create_push_stream(None).unwrap();
    let audio_cfg = AudioConfig::from_stream_input(audio_stream).unwrap();
    let mut speech_cfg = SpeechConfig::from_subscription("c54659a7b47a41009a245526246fb377", "eastasia").unwrap();
    speech_cfg.set(PropertyId::SpeechServiceConnectionRecoLanguage, "en-US").unwrap();
    let mut speech_reco = SpeechRecognizer::from_config(speech_cfg, audio_cfg).unwrap();

//    let f1 = speech_reco.connect_recognized(None).for_each(|x| {
//        let r = x.result().unwrap();
//        println!("recognized reason: {:?}, text: {}", r.reason().unwrap(), r.text().unwrap());
//        Ok(())
//    });

    let s1 = speech_reco.connect_recognizing(None);
    let s2 = speech_reco.connect_recognized(None);

    fn print_event(e: RecognitionResultEvent) {
        let r = e.result().unwrap();
        debug!("session: {}, id: {}, reason: {:?}, e-offset: {}, r-offset: {}, duration: {:?}, text: {}",
               e.session_id().unwrap(),
               r.id().unwrap(),
               r.reason().unwrap(),
               e.offset().unwrap(),
               r.offset().unwrap(),
               r.duration().unwrap(),
               r.text().unwrap());
    }

    let f1 = s1.merge(s2).for_each(|x| {
        match x {
            MergedItem::First(x) => {
                print_event(x);
            }
            MergedItem::Second(x) => {
                print_event(x);
            }
            MergedItem::Both(x, y) => {
                print_event(x);
                print_event(y);
                debug!("both!");
            }
        }
        Ok(())
    });

    let mut r = tokio::runtime::Runtime::new().unwrap();

    r.block_on(speech_reco.start_continuous_recognition().unwrap()).unwrap();

    for _ in 0..2 {
        let mut f = File::open("/home/luyi/whatstheweatherlike.wav").expect("file not found");
        let mut buff = [0u8; 10 * 1024];
        loop {
            let size = f.read(&mut buff).expect("can not read file");
            if size > 0 {
                sink.write(&buff[..size]).unwrap();
                debug!("wrote {} bytes", size);
            }
            if size < buff.len() {
                break;
            }
        }
        for _ in 0..1024 {
            sink.write(&vec![0; 8]).unwrap();
        }
    }

    sink.close().unwrap();
    r.block_on(f1).unwrap();

    sleep(Duration::from_secs(100));
    info!("done");
}
