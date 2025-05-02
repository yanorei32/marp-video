use std::path::PathBuf;

use anyhow::Context;
use futures::future::BoxFuture;
use tokio::task::JoinError;

use crate::application::*;
use crate::environment::Environment;
use crate::event::Event;
use crate::event_parser::*;

struct SoundFuture {
    duration: BoxFuture<'static, Result<anyhow::Result<f64>, JoinError>>,
    path: PathBuf,
    volume: f32,
}

#[derive(Debug, Clone)]
pub struct Sound {
    pub duration: f64,
    pub path: PathBuf,
    pub volume: f32,
}

pub async fn prepare(
    env: &Environment,
    events: &[Event<RawVoiceEvent, RawFgSoundEvent>],
) -> anyhow::Result<Vec<Event<Sound, Sound>>> {
    let mut event_future: Vec<Event<SoundFuture, SoundFuture>> = vec![];

    for event in events {
        match event {
            Event::Voice(voice) => {
                let filepath = env.voice_cache(&voice.profile, &voice.text);

                let handle = tokio::spawn({
                    let text = voice.text.clone();
                    let profile = voice.profile.clone();
                    let env = env.clone();

                    async move {
                        tts(&env, &profile, &text).await?;

                        let filepath = env.voice_cache(&profile, &text);
                        measure_file_duration(&env, filepath.to_str().unwrap()).await
                    }
                });

                event_future.push(Event::Voice(SoundFuture {
                    path: filepath,
                    volume: 100.0,
                    duration: Box::pin(handle),
                }));
            }
            Event::SoundEffect(se) => {
                let handle = tokio::spawn({
                    let env = env.clone();
                    let path = se.path.clone();

                    async move { measure_file_duration(&env, &path.to_str().unwrap()).await }
                });

                event_future.push(Event::SoundEffect(SoundFuture {
                    path: se.path.clone(),
                    volume: se.volume,
                    duration: Box::pin(handle),
                }));
            }
            Event::MVBGMMarker { path, volume } => {
                event_future.push(Event::MVBGMMarker {
                    path: path.clone(),
                    volume: volume.clone(),
                });
            }
            Event::BlankMs(duration) => {
                event_future.push(Event::BlankMs(*duration));
            }
            Event::IPageMarker { path } => {
                event_future.push(Event::IPageMarker { path: path.clone() });
            }
            Event::MPageMarker { marp_page_nth } => {
                event_future.push(Event::MPageMarker {
                    marp_page_nth: *marp_page_nth,
                });
            }
            Event::CPageMarker { color } => {
                event_future.push(Event::CPageMarker {
                    color: color.clone(),
                });
            }
        }
    }

    let mut events: Vec<Event<Sound, Sound>> = vec![];

    for event in event_future {
        match event {
            Event::Voice(sound) => {
                events.push(Event::Voice(Sound {
                    duration: sound
                        .duration
                        .await
                        .unwrap()
                        .with_context(|| "Audio Asset Generator")?,
                    path: sound.path,
                    volume: sound.volume,
                }));
            }
            Event::SoundEffect(sound) => {
                events.push(Event::SoundEffect(Sound {
                    duration: sound
                        .duration
                        .await
                        .unwrap()
                        .with_context(|| "Audio Asset Generator")?,
                    path: sound.path,
                    volume: sound.volume,
                }));
            }
            Event::MVBGMMarker { path, volume } => {
                events.push(Event::MVBGMMarker {
                    path: path.clone(),
                    volume: volume.clone(),
                });
            }
            Event::BlankMs(duration) => {
                events.push(Event::BlankMs(duration));
            }
            Event::IPageMarker { path } => {
                events.push(Event::IPageMarker { path });
            }
            Event::MPageMarker { marp_page_nth } => {
                events.push(Event::MPageMarker { marp_page_nth });
            }
            Event::CPageMarker { color } => {
                events.push(Event::CPageMarker { color });
            }
        }
    }

    Ok(events)
}
