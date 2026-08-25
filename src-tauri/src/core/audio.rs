use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result, bail};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

struct OpenDevice {
    _sink: MixerDeviceSink,
    player: Player,
}

impl OpenDevice {
    fn open() -> Result<Self> {
        let sink = DeviceSinkBuilder::open_default_sink().context("open default audio device")?;
        let player = Player::connect_new(sink.mixer());
        Ok(Self {
            _sink: sink,
            player,
        })
    }
}

/// Holds the output device open only once a sound has actually been played.
///
/// Opening the default sink is a blocking call that keeps a CoreAudio/WASAPI
/// stream alive for as long as it is held. The default `soundType` is `NONE`,
/// so eagerly opening it at startup cost every user launch time and a pinned
/// audio device for a feature most never enable.
#[derive(Default)]
pub struct AudioPlayer {
    device: Option<OpenDevice>,
    /// Set when opening failed, so a machine without an usable output device
    /// does not pay the open cost again on every break.
    unavailable: bool,
}

impl AudioPlayer {
    pub fn play(&mut self, path: &Path, volume: f32) -> Result<()> {
        if self.unavailable {
            bail!("audio device is unavailable");
        }
        let device = match self.device {
            Some(ref mut device) => device,
            None => match OpenDevice::open() {
                Ok(device) => self.device.insert(device),
                Err(error) => {
                    self.unavailable = true;
                    return Err(error);
                }
            },
        };

        let volume = volume.clamp(0.0, 1.0);
        let file = File::open(path).with_context(|| format!("open audio {}", path.display()))?;
        let source = Decoder::new(BufReader::new(file)).context("decode audio")?;
        device.player.stop();
        device.player.set_volume(volume);
        device.player.append(source);
        Ok(())
    }

    /// Drops the output stream. The next `play` reopens it. Does not clear
    /// `unavailable`: a machine that failed to open should not retry forever.
    pub fn close_device(&mut self) {
        self.device = None;
    }
}
