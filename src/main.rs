use std::path::PathBuf;

mod application;
mod asset_preparator;
mod environment;
mod event;
mod event_parser;
mod planner;

use environment::Environment;

use clap::Parser;
use event_parser::DocEvents;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[clap(default_value = "default")]
    profile: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let env = Environment::try_init(&args.input, &args.profile).unwrap();

    // Create project root dir
    env.init_project_root_dir().unwrap();

    // Switch to project root dir
    std::env::set_current_dir(&env.project_root_dir()).unwrap();

    let handle = tokio::spawn({
        let env = env.clone();
        async move { application::marp(&env).await }
    });

    // try parse to md
    let events = DocEvents::parse(&env, &std::fs::read_to_string(&env.md_path()).unwrap()).unwrap();

    println!("{events:#?}");

    let (_page_count, events) =
        tokio::join!(handle, asset_preparator::prepare(&env, &events.events));

    println!("{events:#?}");

    let doc = planner::plan(&env, &events.unwrap()).unwrap();

    application::encode(&env, &doc).await.unwrap();

    Ok(())
}
