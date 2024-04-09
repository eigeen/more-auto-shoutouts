#[cfg(test)]
mod tests {
    #[test]
    fn test_demo() {
        use rodio::source::Source;
        use rodio::{Decoder, OutputStream, Sink};
        use std::fs::File;
        use std::io::BufReader;
        use std::time::Duration;

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let file = BufReader::new(File::open("examples/MyGO!!!!! - 春日影 (MyGO!!!!! ver.).mp3").unwrap());
        let source = Decoder::new(file).unwrap();
        sink.append(source.repeat_infinite());

        sink.sleep_until_end();
    }
}
