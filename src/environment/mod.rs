use std::path::{Path, PathBuf};
use std::collections::HashMap;

use anyhow::Context;
use directories::ProjectDirs;

mod config;
use config::Config;

#[derive(Debug, Clone)]
pub struct Environment {
    config: Config,
    abs_md_path: PathBuf,
    profile: String,
}

impl Environment {
    pub fn try_init(md_path: &Path, profile: &str) -> anyhow::Result<Self> {
        let dirs = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")).with_context(|| {
            "Failed to get application base config directory. Do you have home directory?"
        })?;

        let config_path = dirs.config_dir().join("marp-video.toml");
        let config_path_str = config_path.to_string_lossy();

        let config = Config::try_init(&config_path)
            .with_context(|| format!("Failed to read config file: {config_path_str}"))?;

        let md_path = md_path.canonicalize()?;

        config
            .profile
            .get(profile)
            .ok_or(anyhow::anyhow!("The specified profile ({profile}) is not configured"))?;

        let profile = profile.to_string();

        Ok(Self { config, abs_md_path: md_path, profile })
    }

    pub fn video_width(&self) -> usize {
        self.config.profile[&self.profile].width
    }

    pub fn video_height(&self) -> usize {
        self.config.profile[&self.profile].height
    }

    pub fn marp_additional_args(&self) -> Vec<String> {
        let mut args = vec![];
        args.extend_from_slice(&self.config.dep.marp.global_args);
        args.extend_from_slice(&self.config.profile[&self.profile].marp_args);
        args
    }

    pub fn ffmpeg_additional_args(&self) -> Vec<String> {
        let mut args = vec![];
        args.extend_from_slice(&self.config.dep.ffmpeg.global_args);
        args.extend_from_slice(&self.config.profile[&self.profile].ffmpeg_args);
        args
    }

    pub fn video_container(&self) -> &str {
        &self.config.profile[&self.profile].video_container
    }

    pub fn ffmpeg_bin(&self) -> &str {
        &self.config.dep.ffmpeg.bin
    }

    pub fn tts_additional_args(&self, voice_profile: &str) -> &[String] {
        &self.config.tts[voice_profile].args
    }

    pub fn tts_envs(&self, voice_profile: &str) -> &HashMap<String, String> {
        &self.config.tts[voice_profile].envs
    }

    pub fn tts_is_profile_exists(&self, voice_profile: &str) -> bool {
        self.config.tts.get(voice_profile).is_some()
    }

    pub fn tts_bin(&self, voice_profile: &str) -> &str {
        &self.config.tts[voice_profile].bin
    }

    pub fn marp_envs(&self) -> &HashMap<String, String> {
        &self.config.dep.marp.envs
    }

    pub fn marp_bin(&self) -> &str {
        &self.config.dep.marp.bin
    }

    pub fn ffprobe_bin(&self) -> &str {
        &self.config.dep.ffprobe.bin
    }

    pub fn md_path(&self) -> &Path {
        self.abs_md_path.as_path()
    }

    pub fn md_dir(&self) -> PathBuf {
        let mut path = self.abs_md_path.clone();
        path.pop();
        path
    }

    pub fn cache_root_dir(&self) -> PathBuf {
        if self.config.cache_dir.is_relative() {
            self.md_dir().join(self.config.cache_dir.as_path())
        } else {
            self.config.cache_dir.to_path_buf()
        }
    }

    pub fn voice_cache_dir(&self, voice_profile: &str) -> PathBuf {
        self.cache_root_dir()
            .join(format!("voice_{:x}", md5::compute(voice_profile)))
    }

    pub fn init_voice_cache_dir(&self, voice_profile: &str) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.voice_cache_dir(voice_profile))?;
        Ok(())
    }

    pub fn voice_cache(&self, voice_profile: &str, text: &str) -> PathBuf {
        self.voice_cache_dir(voice_profile)
            .join(format!("{:x}.bin", md5::compute(text)))
    }

    pub fn project_root_dir(&self) -> PathBuf {
        self.cache_root_dir().join(format!(
            "work_{:x}",
            md5::compute(self.abs_md_path.to_str().unwrap())
        ))
    }

    pub fn init_project_root_dir(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.project_root_dir())?;
        Ok(())
    }
}
