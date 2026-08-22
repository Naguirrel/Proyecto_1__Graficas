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
    audio_dir: PathBuf,
    background: Option<PathBuf>,
    power: Option<PathBuf>,
    mp3_count: usize,
}

impl AudioTracks {
    pub fn discover(audio_dir: impl AsRef<Path>) -> Self {
        let audio_dir = audio_dir.as_ref().to_path_buf();
        let paths = collect_mp3_paths(&audio_dir);

        Self {
            audio_dir,
            background: find_background_track(&paths),
            power: find_power_track(&paths),
            mp3_count: paths.len(),
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

    fn missing_message(&self, track: MusicTrack) -> String {
        let expected = match track {
            MusicTrack::Background => "un MP3 cuyo nombre contenga `C fondo`",
            MusicTrack::Power => "un MP3 cuyo nombre termine en `TS`",
        };

        if self.mp3_count == 0 {
            format!(
                "No se encontraron MP3 en {}. Coloca ahi {expected}.",
                self.audio_dir.display()
            )
        } else {
            format!(
                "No se encontro {expected} dentro de {}.",
                self.audio_dir.display()
            )
        }
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
            .map(|mut output| {
                output.log_on_drop(false);
                output
            })
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

        if self.output.is_none() {
            return;
        }

        let Some(path) = self.tracks.path_for(track).map(Path::to_path_buf) else {
            self.warn_missing_track(track);
            if track == MusicTrack::Background || self.active_track != Some(MusicTrack::Background)
            {
                self.stop();
            }
            return;
        };

        self.stop();
        let output = self.output.as_ref().expect("audio output was checked");

        match looped_player(output, &path) {
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
        eprintln!("{}", self.tracks.missing_message(track));
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
        .find(|path| {
            let name = normalized_file_name(path);

            is_mp3(path) && name.contains("c") && name.contains("fondo")
        })
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
                    .is_some_and(|stem| normalized_text(stem).ends_with("ts"))
        })
        .cloned()
}

fn collect_mp3_paths(audio_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    collect_mp3_paths_from(audio_dir, &mut paths);

    paths
}

fn collect_mp3_paths_from(path: &Path, paths: &mut Vec<PathBuf>) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };

    if metadata.is_file() {
        if is_mp3(path) {
            paths.push(path.to_path_buf());
        }
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        collect_mp3_paths_from(&entry.path(), paths);
    }
}

fn is_mp3(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mp3"))
}

fn normalized_file_name(path: &Path) -> String {
    normalized_text(
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
    )
}

fn normalized_text(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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

    #[test]
    fn finds_audio_tracks_with_separators_and_nested_folders() {
        let dir =
            std::env::temp_dir().join(format!("raycasting_audio_nested_{}", std::process::id()));
        let nested_dir = dir.join("musica");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&nested_dir).expect("nested audio dir should be created");
        File::create(nested_dir.join("C_fondo.mp3"))
            .expect("background test file should be created");
        File::create(nested_dir.join("poder-TS.mp3")).expect("power test file should be created");

        let tracks = AudioTracks::discover(&dir);

        assert!(tracks.has_background());
        assert!(tracks.has_power());
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
