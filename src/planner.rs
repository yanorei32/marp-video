use std::path::Path;

use itertools::Itertools;

use crate::asset_preparator::Sound;
use crate::environment::Environment;
use crate::event::Event;

#[derive(Debug, Clone)]
pub struct DocumentChannels {
    pub videos: Vec<String>,
    pub fg_sounds: Vec<String>,
    pub bg_sounds: Vec<String>,
}

fn ffmpeg_escape(path: &Path) -> String {
    // Don't ask me why; it works
    path.to_str()
        .unwrap()
        .replace("\\", "\\\\\\\\")
        .replace(":", "\\\\:")
        .replace("'", "\\\\\\'")
        .replace("=", "\\=")
        .replace(";", "\\;")
        .replace(",", "\\,")
        .replace("[", "\\[")
        .replace("]", "\\]")
}

pub fn plan(env: &Environment, events: &[Event<Sound, Sound>]) -> anyhow::Result<DocumentChannels> {
    let videos = plan_video_stream(env, events);
    let bg_sounds = plan_bg_audio_stream(env, events);
    let fg_sounds = plan_fg_audio_stream(env, events);

    Ok(DocumentChannels {
        videos,
        fg_sounds,
        bg_sounds,
    })
}

fn plan_fg_audio_stream(env: &Environment, events: &[Event<Sound, Sound>]) -> Vec<String> {
    let mut foreground_sound_stream: Vec<String> = vec![];

    for event in events {
        match event {
            Event::Voice(sound) | Event::SoundEffect(sound) => {
                foreground_sound_stream.push(format!(
                    "amovie={},volume={}",
                    ffmpeg_escape(&env.md_dir().join(&sound.path)),
                    sound.volume / 100.0,
                ));
            }
            Event::BlankMs(duration) => {
                let duration = *duration as f32 / 1000.0;
                foreground_sound_stream.push(format!("anullsrc,atrim=duration={duration}"));
            }
            _ => {}
        }
    }

    println!("{foreground_sound_stream:#?}");

    foreground_sound_stream
}

fn plan_bg_audio_stream(env: &Environment, events: &[Event<Sound, Sound>]) -> Vec<String> {
    let mut background_sound_stream: Vec<String> = vec![];

    let silent_len: f64 = events
        .iter()
        .take_while(|e| !e.is_bgm_event())
        .filter_map(|e| match e {
            Event::Voice(s) | Event::SoundEffect(s) => Some(s.duration),
            Event::BlankMs(millis) => Some(*millis as f64 / 1000.0),
            _ => None,
        })
        .sum();

    background_sound_stream.push(format!("anullsrc,atrim=duration={silent_len}"));

    let bgm_positions: Vec<_> = events.iter().positions(|e| e.is_bgm_event()).collect();

    let bgm_refs: Vec<_> = bgm_positions.iter().map(|i| &events[*i]).collect();

    let bgm_durations: Vec<f64> = bgm_positions
        .iter()
        .map(|bgm_pos| {
            let bgm_len: f64 = events[*bgm_pos + 1..]
                .iter()
                .take_while(|e| !e.is_bgm_event())
                .filter_map(|e| match e {
                    Event::Voice(s) | Event::SoundEffect(s) => Some(s.duration),
                    Event::BlankMs(millis) => Some(*millis as f64 / 1000.0),
                    _ => None,
                })
                .sum();

            bgm_len
        })
        .collect();

    for (bref, bdur) in bgm_refs.iter().zip(bgm_durations.iter()) {
        match bref {
            Event::MVBGMMarker { path, volume } => match path {
                Some(path) => {
                    background_sound_stream.push(format!(
                        "amovie={},volume={},aloop=-1:2147483647,atrim=duration={bdur}",
                        ffmpeg_escape(&env.md_dir().join(&path.to_str().unwrap())),
                        *volume / 100.0,
                    ));
                }
                None => {
                    background_sound_stream.push(format!(
                        "anullsrc,volume={},atrim=duration={bdur}",
                        *volume / 100.0,
                    ));
                }
            },
            _ => {}
        }
    }

    println!("{background_sound_stream:#?}");

    background_sound_stream
}

fn plan_video_stream(env: &Environment, events: &[Event<Sound, Sound>]) -> Vec<String> {
    let mut video_stream: Vec<String> = vec![];

    let page_positions: Vec<_> = events.iter().positions(|e| e.is_page()).collect();

    let page_refs: Vec<_> = page_positions.iter().map(|i| &events[*i]).collect();

    let page_durations: Vec<f64> = page_positions
        .iter()
        .map(|page_pos| {
            let page_len: f64 = events[*page_pos + 1..]
                .iter()
                .take_while(|e| !e.is_page())
                .filter_map(|e| match e {
                    Event::Voice(s) | Event::SoundEffect(s) => Some(s.duration),
                    Event::BlankMs(millis) => Some(*millis as f64 / 1000.0),
                    _ => None,
                })
                .sum();

            page_len
        })
        .collect();

    println!("{page_durations:#?}");

    for (pref, pdur) in page_refs.iter().zip(page_durations.iter()) {
        // Drop slides less than one frame
        if *pdur < 60.0 * 2.0 / 1000.0 {
            println!("WARN: Skip frame");
            continue;
        }

        match pref {
            Event::MPageMarker { marp_page_nth } => {
                video_stream.push(format!(
                    "movie=./marp_doc.{marp_page_nth:03},scale={}:{},setsar=1:1,loop=-1:1,trim=duration={pdur}",
                    env.video_width(),
                    env.video_height(),
                ));
            }
            Event::IPageMarker { path } => {
                video_stream.push(format!(
                    "movie={},scale={}:{},setsar=1:1,loop=-1:1,trim=duration={pdur}",
                    ffmpeg_escape(&env.md_dir().join(&path.to_str().unwrap())),
                    env.video_width(),
                    env.video_height(),
                ));
            }
            Event::CPageMarker { color } => {
                video_stream.push(format!(
                    "color=c={color},scale={}:{},setsar=1:1,loop=-1:1,trim=duration={pdur}",
                    env.video_width(),
                    env.video_height(),
                ));
            }
            _ => {}
        }
    }

    println!("{video_stream:#?}");

    video_stream
}
