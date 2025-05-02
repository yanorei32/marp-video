use anyhow::{Context, bail};
use tokio::process::Command;

use crate::environment::Environment;

pub fn count_generated_marp_pages() -> usize {
    glob::glob("marp_doc.*").unwrap().count()
}

pub async fn marp(env: &Environment) -> anyhow::Result<usize> {
    for f in glob::glob("marp_doc.*").unwrap() {
        let p = f.with_context(|| "Get marp doc path")?;
        std::fs::remove_file(p).with_context(|| "Failed to clean-up old marp_doc files")?
    }

    #[rustfmt::skip]
    let mut args = vec![
        "--images", "png",
        "--output", "marp_doc",
    ];

    let additional_args = env.marp_additional_args();

    args.extend(additional_args.iter().map(|v| v.as_str()));
    args.push(&env.md_path().to_str().unwrap());

    println!("Args: marp {}", args.join(" ") );

    let mut child = Command::new(&env.marp_bin())
        .args(args)
        .envs(env.marp_envs())
        .spawn()
        .with_context(|| "Failed to spawn marp")?;

    let exit_code = child
        .wait()
        .await
        .with_context(|| "Failed to get exit-code marp")?;

    if !exit_code.success() {
        match exit_code.code() {
            Some(code) => bail!("Exit code is not 0: {code}"),
            None => bail!("Killed by signal"),
        }
    }

    Ok(count_generated_marp_pages())
}
