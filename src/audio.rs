use std::fs::{self, File};
use std::path::{Path, PathBuf};

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MusicTrack {
    Background,
    Power,
}

impl MusicTrack {
    fn index(self) -> usize {
        match self {
            Self::Background => 0,
            Self::Power => 1,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AudioTracks {
    background: Option<PathBuf>,
    power: Option<PathBuf>,
}

impl AudioTracks {
    pub fn discover(audio_dir: impl AsRef<Path>) -> Self {
        let Ok(entries) = fs::read_dir(audio_dir) else {
            return Self::default();
        };

        let paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| is_mp3(path))
            .collect::<Vec<_>>();

        Self {
            background: find_background_track(&paths),
            power: find_power_track(&paths),
        }
    }

    fn path_for(&self, track: MusicTrack) -> Option<&Path> {
        match track {
            MusicTrack::Background => self.background.as_deref(),
            MusicTrack::Power => self.power.as_deref(),
        }
    }

    #[cfg(test)]
    fn has_background(&self) -> bool {
        self.background.is_some()
    }

    #[cfg(test)]
    fn has_power(&self) -> bool {
        self.power.is_some()
    }
}

pub struct AudioManager {
    output: Option<MixerDeviceSink>,
    player: Option<Player>,
    tracks: AudioTracks,
    active_track: Option<MusicTrack>,
    missing_track_warnings: [bool; 2],
}

impl AudioManager {
    pub fn new(audio_dir: impl AsRef<Path>) -> Self {
        let tracks = AudioTracks::discover(audio_dir);
        let output = DeviceSinkBuilder::open_default_sink()
            .map_err(|error| eprintln!("Audio disabled: {error}"))
            .ok();

        Self {
            output,
            player: None,
            tracks,
            active_track: None,
            missing_track_warnings: [false; 2],
        }
    }

    pub fn update_for_gameplay(&mut self, has_power: bool) {
        if has_power {
            self.play_looped(MusicTrack::Power);
        } else {
            self.play_looped(MusicTrack::Background);
        }
    }

    pub fn stop(&mut self) {
        self.player = None;
        self.active_track = None;
    }

    fn play_looped(&mut self, track: MusicTrack) {
        if self.active_track == Some(track) {
            return;
        }

        self.stop();

        let Some(output) = &self.output else {
            return;
        };
        let Some(path) = self.tracks.path_for(track) else {
            self.warn_missing_track(track);
            return;
        };

        match looped_player(output, path) {
            Ok(player) => {
                self.player = Some(player);
                self.active_track = Some(track);
            }
            Err(error) => {
                eprintln!("Could not play {}: {error}", path.display());
            }
        }
    }

    fn warn_missing_track(&mut self, track: MusicTrack) {
        let index = track.index();

        if self.missing_track_warnings[index] {
            return;
        }

        self.missing_track_warnings[index] = true;
        eprintln!("Audio track not found for {track:?}");
    }
}

fn looped_player(output: &MixerDeviceSink, path: &Path) -> Result<Player, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let source = Decoder::try_from(file)
        .map_err(|error| error.to_string())?
        .repeat_infinite();
    let player = Player::connect_new(output.mixer());
    player.append(source);

    Ok(player)
}

fn find_background_track(paths: &[PathBuf]) -> Option<PathBuf> {
    paths
        .iter()
        .find(|path| is_mp3(path) && normalized_file_name(path).contains("c fondo"))
        .cloned()
}

fn find_power_track(paths: &[PathBuf]) -> Option<PathBuf> {
    paths
        .iter()
        .find(|path| {
            is_mp3(path)
                && path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem.trim().to_lowercase().ends_with("ts"))
        })
        .cloned()
}

fn is_mp3(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mp3"))
}

fn normalized_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .trim()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_named_audio_tracks() {
        let tracks = AudioTracks::discover(test_audio_dir());

        assert!(tracks.has_background());
        assert!(tracks.has_power());
    }

    #[test]
    fn ignores_non_mp3_files_when_discovering_tracks() {
        let paths = vec![
            PathBuf::from("assets/audios/C fondo.wav"),
            PathBuf::from("assets/audios/poder TS.txt"),
        ];

        assert!(find_background_track(&paths).is_none());
        assert!(find_power_track(&paths).is_none());
    }

    #[test]
    fn finds_power_track_by_ts_suffix() {
        let paths = vec![
            PathBuf::from("assets/audios/ambiente.mp3"),
            PathBuf::from("assets/audios/poder TS.mp3"),
        ];

        assert_eq!(find_power_track(&paths), Some(paths[1].clone()));
    }

    fn test_audio_dir() -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("raycasting_audio_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("test audio dir should be created");
        File::create(dir.join("C fondo.mp3")).expect("background test file should be created");
        File::create(dir.join("poder TS.mp3")).expect("power test file should be created");

        dir
    }
}
