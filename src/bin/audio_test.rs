#![allow(warnings)]

use rodio::Sink;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();

    // load the music
    let music = File::open("assets/music/spacelifeNo14.ogg").unwrap();
    let source = rodio::Decoder::new(BufReader::new(music)).unwrap();

    let sink = rodio::Sink::try_new(&handle).unwrap();

    println!("{}", sink.empty());
    sink.append(source);

    println!("{}", sink.empty());
    loop {}
}
