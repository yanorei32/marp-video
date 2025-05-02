use std::path::PathBuf;

use thiserror::Error;

use crate::event::Event;
use crate::environment::Environment;

#[derive(Debug, Clone)]
pub struct RawFgSoundEvent {
    pub path: PathBuf,
    pub volume: f32,
}

#[derive(Debug, Clone)]
pub struct RawVoiceEvent {
    pub text: String,
    pub profile: String,
}

#[derive(Debug, Clone)]
pub enum RawDocElement {
    /// Marp Page Marker
    MarpPageMarker,

    /// Marp Content Marker
    MarpContentMarker,

    /// Marp Video Events
    MVEvent(Event<RawVoiceEvent, RawFgSoundEvent>),
}

fn resource_path(env: &Environment, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.has_root() {
        path
    } else {
        env.md_dir().join(path)
    }
}

impl Event<RawVoiceEvent, RawFgSoundEvent> {
    fn try_from_str(env: &Environment, s: &str) -> Option<Self> {
        match s
            .split_once(':')
            .map(|(key, val)| (key.trim(), val.trim()))?
        {
            ("speak", text) => Some(Event::Voice(RawVoiceEvent {
                text: text.to_owned(),
                profile: "default".to_string(),
            })),
            ("speak_in", text) => {
                let (profile, text) = text
                    .split_once(':')
                    .map(|(key, val)| (key.trim(), val.trim()))?;

                Some(Event::Voice(RawVoiceEvent {
                    text: text.to_owned(),
                    profile: profile.to_string(),
                }))
            }
            ("blank", text) => Some(Event::BlankMs(text.parse().ok()?)),
            ("bgcolor", color) => Some(Event::CPageMarker {
                color: color.to_string(),
            }),
            ("bgimage", image) => Some(Event::IPageMarker {
                path: resource_path(env, &image.replace("path=", "")),
            }),
            ("sound_effect", properties) => {
                let mut path = None;
                let mut volume = 100.0f32;

                for property in properties.split(',') {
                    let (key, value) = property
                        .split_once('=')
                        .map(|(key, val)| (key.trim(), val.trim()))?;

                    match key {
                        "path" => {
                            path = Some(value.to_string());
                        }
                        "volume" => {
                            volume = value.parse().ok()?;
                        }
                        _ => None?,
                    }
                }


                Some(Event::SoundEffect(RawFgSoundEvent {
                    path: resource_path(env, &path?),
                    volume,
                }))
            }
            ("bgm", properties) => {
                let mut path = None;
                let mut volume = 100.0f32;

                for property in properties.split(',') {
                    let (key, value) = property
                        .split_once('=')
                        .map(|(key, val)| (key.trim(), val.trim()))?;

                    match key {
                        "path" => {
                            path = Some(value.to_string());
                        }
                        "volume" => {
                            volume = value.parse().ok()?;
                        }
                        _ => None?,
                    }
                }

                let path = path?;

                let path = if path == "none" {
                    None
                } else {
                    Some(resource_path(env, &path))
                };

                Some(Event::MVBGMMarker{
                    path,
                    volume,
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocEvents {
    pub events: Vec<Event<RawVoiceEvent, RawFgSoundEvent>>,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Markdown parse error: {:?}", .0)]
    Markdown(markdown::message::Message),

    #[error("Frontmatter is not exist")]
    FrontmatterIsNotExist,

    #[error("Invalid frontmatter YAML: {:?}", .0)]
    Frontmatter(serde_yaml::Error),

    #[error("Non marp document")]
    NonMarpDocument,

    #[error("Non marpVideo document")]
    NonMarpVideoDocument,

    #[error("First element is not page")]
    FirstElementIsNotPage,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Frontmatter {
    marp_video: Option<bool>,
    marp: Option<bool>,

    #[serde(default)]
    heading_divider: u8,
}

impl DocEvents {
    pub fn parse(env: &Environment, s: &str) -> Result<Self, ParseError> {
        use markdown::{mdast::Node, Constructs, ParseOptions};

        let md_ast = markdown::to_mdast(
            s,
            &ParseOptions {
                constructs: Constructs {
                    frontmatter: true,
                    ..Constructs::gfm()
                },
                ..ParseOptions::default()
            },
        )
        .map_err(ParseError::Markdown)?;

        let md_nodes = md_ast.children().ok_or(ParseError::FrontmatterIsNotExist)?;
        let first_node = md_nodes.first().ok_or(ParseError::FrontmatterIsNotExist)?;

        let Node::Yaml(yaml) = first_node else {
            return Err(ParseError::FrontmatterIsNotExist);
        };

        let frontmatter: Frontmatter =
            serde_yaml::from_str(&yaml.value).map_err(ParseError::Frontmatter)?;

        if frontmatter.marp != Some(true) {
            return Err(ParseError::NonMarpDocument);
        }

        if frontmatter.marp_video != Some(true) {
            return Err(ParseError::NonMarpVideoDocument);
        }

        let raw_document =
            md_nodes_to_raw_doc_elements(env, &md_nodes, frontmatter.heading_divider);

        let strctured_doc = parse_page_structure(&raw_document);

        if !strctured_doc.first().unwrap().is_page() {
            return Err(ParseError::FirstElementIsNotPage);
        }

        Ok(Self {
            events: strctured_doc,
        })
    }
}

fn md_nodes_to_raw_doc_elements(
    env: &Environment,
    nodes: &[markdown::mdast::Node],
    heading_divider: u8,
) -> Vec<RawDocElement> {
    use markdown::mdast::{Heading, Html, Node};

    let mut raw_document = vec![];

    for node in nodes.iter().skip(1) {
        match node {
            Node::ThematicBreak(_) => {
                raw_document.push(RawDocElement::MarpPageMarker);
            }

            Node::Heading(Heading { depth, .. }) if *depth <= heading_divider => {
                raw_document.push(RawDocElement::MarpPageMarker);
            }

            Node::Html(Html { value, .. })
                if value.starts_with("<!--mv") && value.ends_with("-->") =>
            {
                let marpv_syntax = &value["<!--mv".len()..value.len() - "-->".len()];

                for marpv_line in marpv_syntax.split('\n') {
                    if let Some(event) =
                        Event::<RawVoiceEvent, RawFgSoundEvent>::try_from_str(env, marpv_line)
                    {
                        raw_document.push(RawDocElement::MVEvent(event));
                    }
                }
            }

            Node::Html(Html { value, .. })
                if value.starts_with("<!--") || value.starts_with("<style>") =>
            {
                if value.contains("headingDivider") {
                    panic!("[ERROR] headingDivider comment is not supported. Please write that in frontmatter.");
                }
            }

            Node::Html(_) => {
                raw_document.push(RawDocElement::MarpContentMarker);
            }

            Node::Blockquote(_)
            | Node::FootnoteDefinition(_)
            | Node::MdxJsxTextElement(_)
            | Node::MdxJsxFlowElement(_)
            | Node::List(_)
            | Node::MdxjsEsm(_)
            | Node::Break(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Delete(_)
            | Node::Emphasis(_)
            | Node::MdxTextExpression(_)
            | Node::FootnoteReference(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::Link(_)
            | Node::LinkReference(_)
            | Node::Strong(_)
            | Node::Text(_)
            | Node::Code(_)
            | Node::Math(_)
            | Node::MdxFlowExpression(_)
            | Node::Heading(_)
            | Node::Table(_)
            | Node::Definition(_)
            | Node::Paragraph(_) => {
                raw_document.push(RawDocElement::MarpContentMarker);
            }

            Node::Root(_) => panic!("What's Root!?"),
            Node::Toml(_) | Node::Yaml(_) => panic!("Markdown Toml/Yaml Position Error"),
            Node::TableCell(_) | Node::TableRow(_) | Node::ListItem(_) => {
                panic!("Markdown structual Error")
            }
        }
    }

    raw_document
}

fn parse_page_structure(elements: &[RawDocElement]) -> Vec<Event<RawVoiceEvent, RawFgSoundEvent>> {
    let mut events = vec![];

    let mut seen_some_marp_page = false;
    let mut marp_page_nth = 0;

    for element in elements {
        match element {
            RawDocElement::MarpContentMarker if !seen_some_marp_page => {
                seen_some_marp_page = true;
                marp_page_nth += 1;
                events.push(Event::MPageMarker { marp_page_nth });
            }
            RawDocElement::MarpContentMarker => {
                // do nothing
            }
            RawDocElement::MarpPageMarker => {
                seen_some_marp_page = true;
                marp_page_nth += 1;
                events.push(Event::MPageMarker { marp_page_nth });
            }
            RawDocElement::MVEvent(event) => {
                events.push(event.clone());
            }
        }
    }

    events
}
