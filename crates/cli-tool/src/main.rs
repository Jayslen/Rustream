use clap::{Parser, ValueEnum};
use dialoguer::{Confirm, Input};
use serde::Deserialize;
use std::{fs, path::PathBuf};
use uuid::Uuid;

mod database;
mod filesystem;
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    // Actions
    action: Actions,

    // Media type arg (movie or series)
    #[arg(value_enum)]
    media_type: MediaType,

    #[arg(short = 'i', long, help = "The IMDB ID of the media")]
    media_id: Option<String>,

    #[arg(short = 'c', long = "covers")]
    cover_img_dir: Option<PathBuf>,

    #[arg(short = 'p', long = "posters")]
    poster_img_dir: Option<PathBuf>,

    #[arg(short = 'm', long = "media")]
    media_path: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Actions {
    Add,
    Mod,
    Del,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum MediaType {
    Movie,
    Series,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BodyResponse {
    title: String,
    year: String,
    rated: String,
    released: String,
    runtime: String,
    genre: String,
    director: String,
    writer: String,
    actors: String,
    plot: String,
    language: String,
    country: String,
    awards: String,
    poster: String,
    ratings: Vec<Rating>,
    metascore: String,

    #[serde(rename = "imdbRating")]
    imdb_rating: String,

    #[serde(rename = "imdbVotes")]
    imdb_votes: String,

    #[serde(rename = "imdbID")]
    imdb_id: String,

    #[serde(rename = "Type")]
    media_type: String,

    // dvd: String,
    box_office: String,
    production: String,
    website: String,
    response: String,
}

#[derive(Debug, Deserialize)]
struct Rating {
    #[serde(rename = "Source")]
    source: String,

    #[serde(rename = "Value")]
    value: String,
}

struct MediaEntry {
    uuid: Uuid,
    poster_path: PathBuf,
    cover_path: PathBuf,
    media_type: MediaType,
    media_path: PathBuf,
}

impl MediaEntry {
    fn new(
        poster_path: PathBuf,
        cover_path: PathBuf,
        media_type: MediaType,
        media_path: PathBuf,
    ) -> Self {
        Self {
            poster_path,
            cover_path,
            media_type,
            media_path,
            uuid: Uuid::new_v4(),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.action {
        Actions::Add => {
            let mut media_name = if let Some(media_id) = &cli.media_id {
                media_id.clone()
            } else {
                Input::new()
                    .with_prompt("Enter the IMDB ID of the media \n")
                    .interact_text()
                    .unwrap()
            };

            let mut selected_media: Option<BodyResponse> = None;

            while match get_media_result(&media_name) {
                Ok(media_response) => {
                    let confirm = true;
                    if confirm {
                        println!(
                            "Media confirmed: {} ({})",
                            media_response.title, media_response.year
                        );
                        selected_media = Some(media_response);
                        false // Exit
                    } else {
                        media_name = Input::new()
                            .with_prompt("Enter the IMDB ID of the media")
                            .interact_text()
                            .unwrap();
                        true // Continue
                    }
                }
                Err(_) => {
                    println!("Media not found.");
                    media_name = Input::new()
                        .with_prompt("Enter the IMDB ID of the media")
                        .interact_text()
                        .unwrap();
                    true // Continue the loop
                }
            } {}

            let covers_dir = get_path_value(cli.cover_img_dir, "covers directory");
            let posters_dir = get_path_value(cli.poster_img_dir, "posters directory");
            let media_path = get_path_value(cli.media_path, "media file");

            let media_entry = MediaEntry::new(posters_dir, covers_dir, cli.media_type, media_path);

            let media_details =
                selected_media.expect("Media details should be available at this point");
            let assets = filesystem::add_media(&media_entry, &media_details)
                .expect("Could not add media files to the filesystem");

            database::add_media(&media_entry, &media_details, &assets)
                .expect("Could not add media metadata to the database");

            println!("Media added successfully: {:?}", assets);
        }
        Actions::Mod => {
            println!("Modifying a movie...");
        }
        Actions::Del => {
            println!("Deleting a movie...");
        }
    }
}

// Helper to get value interactive in case is not present
fn get_path_value(path: Option<PathBuf>, path_msg: &str) -> PathBuf {
    match path {
        Some(p) => p.clone(),
        None => loop {
            let input = Input::<String>::new()
                .with_prompt(format!("Enter the path to the {}", path_msg))
                .interact_text()
                .unwrap();

            match fs::exists(&input) {
                Ok(exists) => {
                    if exists {
                        return PathBuf::from(input);
                    } else {
                        println!("The path does not exist. Please try again.");
                    }
                }
                Err(e) => {
                    println!("Error checking the path: {}. Please try again.", e);
                }
            }
        },
    }
}

fn get_media_result(title: &String) -> Result<BodyResponse, Box<dyn std::error::Error>> {
    let api_key = std::env::var("API_KEY")?;
    let api = format!(
        "https://www.omdbapi.com/?{}={}&apikey={}",
        if title.starts_with("tt") { "i" } else { "t" },
        title,
        api_key
    );

    let res = ureq::get(api)
        .call()?
        .body_mut()
        .read_json::<BodyResponse>()?;

    Ok(res)
}

fn confirm_media(media_response: &BodyResponse) -> bool {
    let confirm = Confirm::new()
        .with_prompt(format!(
            "{} ({}) - Confirm?",
            media_response.title, media_response.year
        ))
        .default(true)
        .interact()
        .unwrap();

    confirm
}
