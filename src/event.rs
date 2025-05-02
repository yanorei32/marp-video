use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Event<Voice, FgSound> {
    Voice(Voice),

    SoundEffect(FgSound),

    BlankMs(usize),

    /// Marp Video BGM Marker
    MVBGMMarker {
        path: Option<PathBuf>,
        volume: f32,
    },

    /// Marp Video Virtual Page (Image)
    IPageMarker {
        path: PathBuf,
    },

    /// Marp Video Virtual Page (Color)
    CPageMarker {
        color: String,
    },

    /// Marp Page
    MPageMarker {
        marp_page_nth: usize,
    },
}

impl<Voice, FgSound> Event<Voice, FgSound> {
    pub fn is_page(&self) -> bool {
        match self {
            Self::IPageMarker { .. } | Self::CPageMarker { .. } | Self::MPageMarker { .. } => true,
            _ => false,
        }
    }

    pub fn is_bgm_event(&self) -> bool {
        match self {
            Self::MVBGMMarker { .. } => true,
            _ => false,
        }
    }
}
