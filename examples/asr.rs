use env_logger;
use futures::Stream;
use log::{debug, info};
use microsoft_speech::{
    audio::AudioConfig,
    recognizer::{events::RecognitionResultEvent, RecognitionResult, SpeechRecognizer},
    PropertyId, SpeechConfig,
};
use std::{env, time::Duration, thread::sleep};

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "trace");
    env_logger::init();
    info!("Start ASR test...");

    let mut sc =
        SpeechConfig::from_subscription("YourSubscriptionKey", "YourServiceRegion").unwrap();
    sc.set(PropertyId::SpeechServiceConnectionRecoLanguage, "zh-CN")
        .unwrap();

    let ac = AudioConfig::from_wav_file_input("chinese_test.wav").unwrap();
    let mut recognizer = SpeechRecognizer::from_config(sc, Some(ac)).unwrap();
    let s1 = recognizer.connect_recognizing(None);
    let s2 = recognizer.connect_recognized(None);
    let f1 = s1.select(s2).for_each(|x| {
        print_event(x);
        Ok(())
    });
    recognizer.start_continuous_recognition().unwrap();

    let mut r = tokio::runtime::Runtime::new().unwrap();

    r.block_on(f1).unwrap();
    sleep(Duration::from_secs(100));
    info!("done");
}

fn print_event(e: RecognitionResultEvent<RecognitionResult>) {
    let r = e.result().unwrap();
    debug!(
        "session: {}, id: {}, reason: {:?}, e-offset: {}, r-offset: {}, duration: {:?}, text: {}",
        e.session_id().unwrap(),
        r.id().unwrap(),
        r.reason().unwrap(),
        e.offset().unwrap(),
        r.offset().unwrap(),
        r.duration().unwrap(),
        r.text().unwrap()
    );
}
