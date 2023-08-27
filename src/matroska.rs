use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use std::fmt::{self};

use crate::{command::Command, eyre, Context, Result};

#[derive(Debug)]
pub struct Matroska {
    audios: Vec<MatroskaModel>,
    videos: Vec<MatroskaModel>,
}

impl Matroska {
    pub fn new(path: &str) -> Result<Self> {
        let output = Command::get_infos(path)?;
        let infos = serde_json::from_str::<MatroskaModelWrapper>(&output)
            .context(eyre!("Cannot deserialize mkv infos for '{path}'"))?
            .tracks;

        // Collect audios and videos
        let (audios, videos): (Vec<MatroskaModel>, Vec<MatroskaModel>) = infos
            .into_iter()
            .filter(|f| !matches!(f.track, MatroskaModelTrack::Subtitles))
            .partition(|f| matches!(f.track, MatroskaModelTrack::Audio));

        Ok(Self { audios, videos })
    }

    /// Resolution
    pub fn get_resolution(&self) -> Result<String> {
        Ok(format!(
            "{}",
            self.videos
                .get(0)
                .ok_or_else(|| eyre!("Cannot fetch mkv resolution"))?
                .properties
                .resolution
        ))
    }

    /// Audio codecs
    pub fn get_audio_codecs(&self) -> Result<String> {
        let codecs: Vec<&str> = self
            .audios
            .iter()
            .map(|f| match f.codec.as_str() {
                "E-AC-3" => "EAC3",
                "AC-3" => "AC3",
                _ => f.codec.as_str(),
            })
            .unique()
            .collect();

        match codecs.is_empty() {
            true => Err(eyre!("Cannot detect audio codecs")),
            false => Ok(codecs.join(".")),
        }
    }

    /// Video codecs
    pub fn get_video_codecs(&self) -> Result<String> {
        let codecs: Vec<&str> = self
            .videos
            .iter()
            .flat_map(|f| {
                f.codec
                    .split('/')
                    .map(|f| match f.to_lowercase() {
                        c if c.starts_with("h.264") => "h264",
                        c if c.starts_with("avc") => "h264",
                        c if c.starts_with("h.265") => "h265",
                        c if c.starts_with("hevc") => "h265",
                        c if c.starts_with("x264") => "x264",
                        c if c.starts_with("x.264") => "x264",
                        c if c.starts_with("x265") => "x265",
                        c if c.starts_with("x.265") => "x265",
                        c if c.starts_with("vp9") => "VP9",
                        c if c.starts_with("av1") => "AV1",
                        _ => "",
                    })
                    .collect::<Vec<&str>>()
            })
            .unique()
            .filter(|f| !f.is_empty())
            .collect();

        match codecs.is_empty() {
            true => Err(eyre!("Cannot detect video codecs")),
            false => Ok(codecs.join(".")),
        }
    }
}

#[derive(Deserialize)]
struct MatroskaModelWrapper {
    tracks: Vec<MatroskaModel>,
}

#[derive(Deserialize, Debug)]
struct MatroskaModel {
    codec: String,
    #[serde(rename(deserialize = "type"))]
    track: MatroskaModelTrack,
    properties: MatroskaModelProperties,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum MatroskaModelTrack {
    Audio,
    Video,
    Subtitles,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum MatroskaDimension {
    P480,
    P576,
    P720,
    P1080,
    P1440,
    P2160,
    P4320,
    None,
}

#[derive(Deserialize, Debug)]
struct MatroskaModelProperties {
    #[serde(
        rename = "pixel_dimensions",
        default = "default_matroska_dimension",
        deserialize_with = "deserialize_matroska_dimension"
    )]
    resolution: MatroskaDimension,
}

fn default_matroska_dimension() -> MatroskaDimension {
    MatroskaDimension::None
}

fn deserialize_matroska_dimension<'de, D>(deserializer: D) -> Result<MatroskaDimension, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    let opt: String = opt.unwrap_or_default();
    match opt {
        c if c.starts_with("640") || c.ends_with("480") => Ok(MatroskaDimension::P480),
        c if c.starts_with("720") || c.ends_with("576") => Ok(MatroskaDimension::P576),
        c if c.starts_with("1280") || c.ends_with("720") => Ok(MatroskaDimension::P720),
        c if c.starts_with("1920") || c.ends_with("1080") => Ok(MatroskaDimension::P1080),
        c if c.starts_with("2560") || c.ends_with("1440") => Ok(MatroskaDimension::P1440),
        c if c.starts_with("3840") || c.ends_with("2160") => Ok(MatroskaDimension::P2160),
        c if c.starts_with("7680") || c.ends_with("4320") => Ok(MatroskaDimension::P4320),
        _ => Ok(MatroskaDimension::None),
    }
}

impl fmt::Display for MatroskaDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::P480 => write!(f, "480p"),
            Self::P576 => write!(f, "576p"),
            Self::P720 => write!(f, "720p"),
            Self::P1080 => write!(f, "1080p"),
            Self::P1440 => write!(f, "1440p"),
            Self::P2160 => write!(f, "2160p"),
            Self::P4320 => write!(f, "4320p"),
            Self::None => write!(f, ""),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum MatroskaSource {
    Ppv,
    TS,
    Cam,
    HDcam,
    Scr,
    DVDScr,
    TVRip,
    HDlight,
    HDRip,
    VODRip,
    WEBRip,
    WEBDl,
    MiniHD,
    BluRay,
    UHDBluRay,
    Remux,
    None,
}

impl From<&str> for MatroskaSource {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "ppv" => Self::Ppv,
            "ts" => Self::TS,
            "cam" => Self::Cam,
            "hdcam" => Self::HDcam,
            "scr" => Self::Scr,
            "dvdscr" | "dvd" => Self::DVDScr,
            "tvrip" | "tv" => Self::TVRip,
            "hdlight" => Self::HDlight,
            "hdrip" => Self::HDRip,
            "vodrip" | "vod" => Self::VODRip,
            "webrip" => Self::WEBRip,
            "webdl" | "web-dl" | "web" => Self::WEBDl,
            "minihd" | "microhd" => Self::MiniHD,
            "br" | "brrip" | "blu-ray" | "bluray" => Self::BluRay,
            "uhdbr" | "uhdblu-ray" | "uhdbluray" => Self::UHDBluRay,
            "bruhd" | "blu-rayuhd" | "blurayuhd" => Self::UHDBluRay,
            "remux" => Self::Remux,
            _ => Self::None,
        }
    }
}

impl fmt::Display for MatroskaSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Ppv => write!(f, "PPV"),
            Self::TS => write!(f, "TC"),
            Self::Cam => write!(f, "CAM"),
            Self::HDcam => write!(f, "HDCAM"),
            Self::Scr => write!(f, "SCR"),
            Self::DVDScr => write!(f, "DVDScr"),
            Self::TVRip => write!(f, "TVRip"),
            Self::HDlight => write!(f, "HDLight"),
            Self::HDRip => write!(f, "HDRip"),
            Self::VODRip => write!(f, "VODRip"),
            Self::WEBRip => write!(f, "WEBRip"),
            Self::WEBDl => write!(f, "WEBDL"),
            Self::MiniHD => write!(f, "MiniHD"),
            Self::BluRay => write!(f, "BluRay"),
            Self::UHDBluRay => write!(f, "UHDBluRay"),
            Self::Remux => write!(f, "Remux"),
            Self::None => write!(f, ""),
        }
    }
}

pub enum MatroskaLang {
    MULTi,
    VOSTfr,
    Vff,
    None,
}

impl From<&str> for MatroskaLang {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "multi" => Self::MULTi,
            "vost" | "vostfr" => Self::VOSTfr,
            "vff" => Self::Vff,
            _ => Self::None,
        }
    }
}

// impl TryFrom<&str> for MatroskaLang {
//     type Error = color_eyre::eyre::Error;

//     fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
//         match value.to_lowercase().as_str() {
//             "multi" => Ok(Self::MULTi),
//             "vostfr" => Ok(Self::VOSTFR),
//             "vff" => Ok(Self::VFF),
//             _ => Err(eyre!("")),
//         }
//     }
// }

impl fmt::Display for MatroskaLang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::MULTi => write!(f, "MULTi"),
            Self::VOSTfr => write!(f, "VOSTFR"),
            Self::Vff => write!(f, "VFF"),
            Self::None => write!(f, ""),
        }
    }
}
