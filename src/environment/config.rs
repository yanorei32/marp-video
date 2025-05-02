use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_env_field::EnvField;
use serde::Deserialize;

fn default_ffprobe_bin() -> String {
    String::from("ffprobe")
}

fn default_ffmpeg_bin() -> String {
    String::from("ffmpeg")
}

fn default_marp_bin() -> String {
    String::from("marp")
}

fn default_tts_bin() -> String {
    String::from("marp-video-tts")
}

fn default_video_container() -> String {
    String::from("mp4")
}

fn default_width() -> usize {
    1280
}

fn default_height() -> usize {
    720
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Ffmpeg {
    #[serde(default = "default_ffmpeg_bin")]
    pub bin: String,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub global_args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Marp {
    #[serde(default = "default_marp_bin")]
    pub bin: String,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub global_args: Vec<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub envs: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Ffprobe {
    #[serde(default = "default_ffprobe_bin")]
    pub bin: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Tts {
    #[serde(default = "default_tts_bin")]
    pub bin: String,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub args: Vec<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub envs: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Dependencies {
    pub ffprobe: Ffprobe,
    pub ffmpeg: Ffmpeg,
    pub marp: Marp,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Profile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ffmpeg_args: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub marp_args: Vec<String>,

    #[serde(default = "default_video_container")]
    pub video_container: String,

    #[serde(default = "default_width")]
    pub width: usize,

    #[serde(default = "default_height")]
    pub height: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub cache_dir: EnvField<PathBuf>,
    pub dep: Dependencies,
    pub tts: HashMap<String, Tts>,
    pub profile: HashMap<String, Profile>,
}

impl Config {
    pub fn try_init(path: &Path) -> anyhow::Result<Self> {
        let cfg = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&cfg)?)
    }
}
