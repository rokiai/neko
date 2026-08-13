use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

pub struct AudioPlayer {
    _sink: MixerDeviceSink,
    player: Player,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let sink = DeviceSinkBuilder::open_default_sink().context("open default audio device")?;
        let player = Player::connect_new(sink.mixer());
        Ok(Self {
            _sink: sink,
            player,
        })
    }

    pub fn play(&mut self, path: &Path, volume: f32) -> Result<()> {
        let volume = volume.clamp(0.0, 1.0);
        let file = File::open(path).with_context(|| format!("open audio {}", path.display()))?;
        let source = Decoder::new(BufReader::new(file)).context("decode audio")?;
        self.player.stop();
        self.player.set_volume(volume);
        self.player.append(source);
        Ok(())
    }
}
