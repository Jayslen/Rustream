use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(value_enum)]
    // Actions
    pub action: Actions,

    // Media type arg (movie or series)
    #[arg(value_enum)]
    pub media_type: MediaType,

    #[arg(short = 'i', long, help = "The IMDB ID of the media")]
    pub media_id: Option<String>,

    #[arg(short = 'c', long = "covers")]
    pub cover_img_dir: Option<PathBuf>,

    #[arg(short = 'p', long = "posters")]
    pub poster_img_dir: Option<PathBuf>,

    #[arg(short = 'm', long = "media")]
    pub media_path: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Actions {
    Add,
    Mod,
    Del,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum MediaType {
    Movie,
    Series,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BodyResponse {
    pub title: String,
    pub year: String,
    // rated: String,
    // released: String,
    // runtime: String,
    pub genre: String,
    pub director: String,
    pub writer: String,
    pub actors: String,
    pub plot: String,
    // language: String,
    // country: String,
    // awards: String,
    // poster: String,
    // ratings: Vec<Rating>,
    // metascore: String,

    // #[serde(rename = "imdbRating")]
    // imdb_rating: String,

    // #[serde(rename = "imdbVotes")]
    // imdb_votes: String,

    // #[serde(rename = "imdbID")]
    // imdb_id: String,
    // #[serde(rename = "Type")]
    // media_type: String,

    // dvd: String,
    // box_office: String,
    // production: String,
    // website: String,
    // response: String,
}

// #[derive(Debug, Deserialize)]
// struct Rating {
//     #[serde(rename = "Source")]
//     source: String,
//
//     #[serde(rename = "Value")]
//     value: String,
// }

pub struct MediaEntry {
    pub uuid: Uuid,
    pub media_type: MediaType,
    pub media_path: PathBuf,
}

impl MediaEntry {
    pub fn new(media_type: MediaType, media_path: PathBuf) -> Self {
        Self {
            media_type,
            media_path,
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub struct MediaAsset {
    pub id: Uuid,
    pub filename: String,
    pub source_path: PathBuf,
    pub(crate) asset_type: AssetType,
}

#[derive(Debug)]
pub(crate) enum AssetType {
    Poster,
    Cover,
}

impl AssetType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Poster => "poster",
            Self::Cover => "cover",
        }
    }
}
