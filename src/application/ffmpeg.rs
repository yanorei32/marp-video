use anyhow::Context;
use tokio::process::Command;

use crate::{environment::Environment, planner::DocumentChannels};

fn to_filter_complex(doc: &DocumentChannels) -> String {
    let filter_complex = String::new();

    let filter_complex = doc
        .fg_sounds
        .iter()
        .enumerate()
        .fold(filter_complex, |acc, (n, v)| format!("{acc}{v}[fga{n}];\n"));

    // generate likes [fga0][fga1][fga2] ...
    let concat_targets =
        (0..doc.fg_sounds.len()).fold(String::new(), |acc, n| format!("{acc}[fga{n}]"));

    let filter_complex = format!(
        "{filter_complex}{concat_targets}concat=n={}:v=0:a=1[fga];\n",
        doc.fg_sounds.len()
    );

    let filter_complex = doc
        .videos
        .iter()
        .enumerate()
        .fold(filter_complex, |acc, (n, v)| format!("{acc}{v}[v{n}];\n"));

    // generate likes [v0][v1][v2] ...
    let concat_targets = (0..doc.videos.len()).fold(String::new(), |acc, n| format!("{acc}[v{n}]"));

    let filter_complex = format!(
        "{filter_complex}{concat_targets}concat=n={}:v=1:a=0[v];\n",
        doc.videos.len()
    );

    let filter_complex = doc
        .bg_sounds
        .iter()
        .enumerate()
        .fold(filter_complex, |acc, (n, v)| format!("{acc}{v}[bga{n}];\n"));

    // generate likes [fga0][fga1][fga2] ...
    let concat_targets =
        (0..doc.bg_sounds.len()).fold(String::new(), |acc, n| format!("{acc}[bga{n}]"));

    let filter_complex = format!(
        "{filter_complex}{concat_targets}concat=n={}:v=0:a=1[bga];\n",
        doc.bg_sounds.len()
    );

    let filter_complex = format!(
        "{filter_complex}[bga][fga]amix[a];\n"
    );

    format!("{filter_complex}\n[v][a]concat=n=1:v=1:a=1")
}

pub async fn encode(env: &Environment, doc: &DocumentChannels) -> anyhow::Result<()> {
    let filter_complex = to_filter_complex(doc);

    let mut a = vec![
        "-nostdin",
        "-hide_banner",
        "-y",
        "-filter_complex",
        &filter_complex,
    ];

    let additional_args = env.ffmpeg_additional_args();
    a.extend(additional_args.iter().map(|s| s.as_str()));

    let filename = format!("output.{}", env.video_container());
    a.push(&filename);

    println!("ffmpeg options: {}", a.join(" "));

    let mut child = Command::new(env.ffmpeg_bin())
        .args(a)
        .spawn()
        .with_context(|| "Failed to spawn ffmpeg")?;

    let exit_code = child
        .wait()
        .await
        .with_context(|| "Failed to get exit-code ffmpeg")?;

    if !exit_code.success() {
        match exit_code.code() {
            Some(code) => return Err(anyhow::anyhow!("Exit code is not 0: {code}")),
            None => return Err(anyhow::anyhow!("Killed by signal")),
        }
    }

    if !std::fs::exists(&filename).with_context(|| "Failed to check file existency")? {
        return Err(anyhow::anyhow!(
            "ffmpeg exit succeed, but output file is not created"
        ));
    }

    Ok(())
}
