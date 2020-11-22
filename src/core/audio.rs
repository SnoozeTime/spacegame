use crate::assets::audio::Audio;
use crate::assets::{AssetManager, Handle};
use crate::event::GameEvent;
use crate::resources::Resources;
use luminance_glfw::GlfwSurface;
use shrev::{EventChannel, ReaderId};
use std::io::{BufReader, Cursor};

pub struct AudioSystem {
    _stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,

    /// Sink for the background music.
    background: rodio::Sink,

    /// Sinks for sound
    sound_sinks: Vec<rodio::Sink>,

    rdr_id: ReaderId<GameEvent>,
}

impl AudioSystem {
    pub fn new(resources: &Resources, channel_nb: usize) -> Result<Self, anyhow::Error> {
        let (stream, handle) = rodio::OutputStream::try_default()?;
        let background = rodio::Sink::try_new(&handle)?;

        let mut sound_sinks = vec![];
        for _ in 0..channel_nb {
            sound_sinks.push(rodio::Sink::try_new(&handle)?);
        }
        let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();

        Ok(Self {
            _stream: stream,
            handle,
            sound_sinks,
            background,
            rdr_id: channel.register_reader(),
        })
    }

    pub fn process(&mut self, resources: &Resources) {
        let channel = resources.fetch::<EventChannel<GameEvent>>().unwrap();
        let audio_manager = resources
            .fetch::<AssetManager<GlfwSurface, Audio>>()
            .unwrap();
        for ev in channel.read(&mut self.rdr_id) {
            match ev {
                GameEvent::PlayBackgroundMusic(name) => {
                    if let Some(asset) = audio_manager.get(&Handle(name.to_string())) {
                        if !self.background.empty() {
                            self.background.stop();
                            self.background = rodio::Sink::try_new(&self.handle)
                                .expect("SHould be able to create new sink");
                        }

                        asset.execute(|audio| {
                            info!("Could load asset");

                            if let Audio::File(content) = audio {
                                self.background.append(
                                    rodio::Decoder::new(BufReader::new(Cursor::new(
                                        content.clone(),
                                    )))
                                    .unwrap(),
                                );

                                self.background.play();
                            }
                        });
                    } else {
                        error!("No asset with name: {}", name);
                    }
                }
                GameEvent::PlaySound(name) => {
                    if let Some(asset) = audio_manager.get(&Handle(name.to_string())) {
                        asset.execute(|audio| {
                            if let Audio::File(content) = audio {
                                // get the first available channel.
                                let sink = self
                                    .sound_sinks
                                    .iter_mut()
                                    .filter(|sink| sink.empty())
                                    .next();
                                if let Some(s) = sink {
                                    s.append(
                                        rodio::Decoder::new(BufReader::new(Cursor::new(
                                            content.clone(),
                                        )))
                                        .unwrap(),
                                    );
                                }
                            }
                        });
                    } else {
                        error!("No asset with name: {}", name);
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn play_background_music(resources: &Resources, name: &str) {
    let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
    channel.single_write(GameEvent::PlayBackgroundMusic(name.to_string()));
}

pub fn play_sound(resources: &Resources, name: &str) {
    let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
    channel.single_write(GameEvent::PlaySound(name.to_string()));
}
