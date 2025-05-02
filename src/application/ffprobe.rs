use std::process::Stdio;

use tokio::process::Command;
use anyhow::{Context, bail};

use crate::Environment;

pub async fn measure_file_duration(env: &Environment, file_path: &str) -> anyhow::Result<f64> {
    #[rustfmt::skip]
    let mut child = Command::new(env.ffprobe_bin())
        .args([
            "-hide_banner",
            "-v", "error",
            "-select_streams", "a:0",
            "-show_entries", "stream=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            file_path,
        ])
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn ffprobe")?;

    let exit_code = child
        .wait()
        .await
        .with_context(|| "Failed to get exit-code ffprobe")?;

    if !exit_code.success() {
        match exit_code.code() {
            Some(code) => bail!("Exit code is not 0: {code}"),
            None => bail!("Killed by signal"),
        }
    }

    let stdout = child
        .wait_with_output()
        .await
        .with_context(|| "Failed to obtain stdout from ffprobe")?
        .stdout;

    let stdout = std::str::from_utf8(&stdout)
        .with_context(|| "Failed to parse ffprobe output as UTF-8")?
        .trim();

    stdout
        .parse()
        .with_context(|| format!("Failed to parse ffprobe output as f64: {}", stdout))
}
