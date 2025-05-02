use anyhow::{bail, Context};
use tokio::process::Command;

use crate::environment::Environment;

pub async fn tts(env: &Environment, voice_profile: &str, text: &str) -> anyhow::Result<String> {
    if !env.tts_is_profile_exists(voice_profile) {
        bail!("Specified voice_profile '{voice_profile}' is not configured");
    }

    let out_path = env.voice_cache(voice_profile, text);

    if std::fs::exists(&out_path).with_context(|| "Checking cached file")? {
        return Ok(out_path.to_str().unwrap().to_string());
    }

    env.init_voice_cache_dir(voice_profile).with_context(|| "Failed to create voice profile directory")?;

    let mut a = vec![text, out_path.to_str().unwrap()];
    let additional_args = env.tts_additional_args(voice_profile);

    a.extend(additional_args.iter().map(|s| s.as_str()));

    let mut child = Command::new(env.tts_bin(voice_profile))
        .args(a)
        .envs(env.tts_envs(voice_profile))
        .spawn()
        .with_context(|| "Failed to spawn tts")?;

    let exit_code = child
        .wait()
        .await
        .with_context(|| "Failed to get exit-code tts")?;

    if !exit_code.success() {
        match exit_code.code() {
            Some(code) => return Err(anyhow::anyhow!("Exit code is not 0: {code}")),
            None => return Err(anyhow::anyhow!("Killed by signal")),
        }
    }

    if !std::fs::exists(&out_path).with_context(|| "Failed to check file existency")? {
        bail!("TTS exit succeed, but output file is not created {voice_profile}「{text}」 {}", out_path.display())
    }

    Ok(out_path.to_str().unwrap().to_string())
}
