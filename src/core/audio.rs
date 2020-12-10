use crate::config::AudioConfig;
use crate::event::GameEvent;
use crate::resources::Resources;
use shrev::{EventChannel, ReaderId};

pub struct AudioSystem {
    current_background: Option<String>,
    rdr_id: ReaderId<GameEvent>,
    config: AudioConfig,
    backend: backend::AudioBackend,
}

impl AudioSystem {
    pub fn new(resources: &Resources, config: AudioConfig) -> Result<Self, anyhow::Error> {
        let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();

        match backend::AudioBackend::new(&config).map(|backend| Self {
            backend,
            config,
            current_background: None,
            rdr_id: channel.register_reader(),
        }) {
            Ok(system) => Ok(system),
            Err(e) => {
                error!("{:?}", e);
                panic!("BOO")
            }
        }
    }

    pub fn process(&mut self, resources: &Resources) {
        let channel = resources.fetch::<EventChannel<GameEvent>>().unwrap();
        for ev in channel.read(&mut self.rdr_id) {
            match ev {
                GameEvent::PlayBackgroundMusic(name) => {
                    self.current_background =
                        self.backend
                            .play_background_music(name, &self.config, resources);
                }
                GameEvent::PlaySound(name) => {
                    self.backend.play_sound(name, resources);
                }
                _ => (),
            }
        }

        // LOOP !
        self.backend.repeat_bg(&self.current_background, resources);
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

mod backend {
    use crate::assets::audio::Audio;
    use crate::assets::{AssetManager, Handle};
    use crate::config::AudioConfig;
    use crate::resources::Resources;
    use std::io::{BufReader, Cursor};

    pub struct AudioBackend {
        _stream: rodio::OutputStream,
        handle: rodio::OutputStreamHandle,

        /// Sink for the background music.
        background: rodio::Sink,

        /// Sinks for sound
        sound_sinks: Vec<rodio::Sink>,
    }

    impl AudioBackend {
        pub fn new(config: &AudioConfig) -> Result<Self, anyhow::Error> {
            let (stream, handle) = rodio::OutputStream::try_default()?;
            let background = rodio::Sink::try_new(&handle)?;
            background.set_volume(config.background_volume as f32 / 100.0);
            let mut sound_sinks = vec![];
            for _ in 0..config.channel_nb {
                sound_sinks.push({
                    let sink = rodio::Sink::try_new(&handle)?;
                    sink.set_volume(config.effects_volume as f32 / 100.0);
                    sink
                });
            }

            Ok(Self {
                _stream: stream,
                handle,
                sound_sinks,
                background,
            })
        }

        pub fn play_background_music(
            &mut self,
            name: &str,
            config: &AudioConfig,
            resources: &Resources,
        ) -> Option<String> {
            let audio_manager = resources.fetch::<AssetManager<Audio>>().unwrap();
            if let Some(asset) = audio_manager.get(&Handle(name.to_string())) {
                if !self.background.empty() {
                    self.background.stop();
                    self.background = rodio::Sink::try_new(&self.handle)
                        .expect("SHould be able to create new sink");
                    self.background
                        .set_volume(config.background_volume as f32 / 100.0);
                }

                asset.execute(|audio| {
                    info!("Could load asset");

                    if let Audio::File(content) = audio {
                        self.background.append(
                            rodio::Decoder::new(BufReader::new(Cursor::new(content.clone())))
                                .unwrap(),
                        );

                        self.background.play();
                    }
                });

                Some(name.to_string())
            } else {
                error!("No asset with name: {}", name);
                None
            }
        }

        pub fn play_sound(&mut self, name: &str, resources: &Resources) {
            let audio_manager = resources.fetch::<AssetManager<Audio>>().unwrap();
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
                                rodio::Decoder::new(BufReader::new(Cursor::new(content.clone())))
                                    .unwrap(),
                            );
                        }
                    }
                });
            } else {
                error!("No asset with name: {}", name);
            }
        }

        pub fn repeat_bg(&mut self, current_bg: &Option<String>, resources: &Resources) {
            if let Some(ref bg) = current_bg {
                if self.background.empty() {
                    let audio_manager = resources.fetch::<AssetManager<Audio>>().unwrap();

                    if let Some(asset) = audio_manager.get(&Handle(bg.clone())) {
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
                    }
                }
            }
        }
    }
}
