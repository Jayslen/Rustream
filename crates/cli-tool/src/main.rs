use clap::{Parser, Subcommand, ValueEnum};
use dialoguer::{Confirm, Input};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    // Actions
    action: Actions,

    // Media type arg (movie or series)
    #[arg(value_enum)]
    media_type: Media,

    #[arg(short = 'i', long, help = "The IMDB ID of the media")]
    media_id: Option<String>,

    #[arg(short = 'c', long = "covers")]
    cover_img_dir: Option<PathBuf>,

    #[arg(short = 'p', long = "posters")]
    poster_img_dir: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Actions {
    Add,
    Mod,
    Del,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Media {
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

            let mut selected_media: BodyResponse;

            loop {
                if let Ok(media_response) = get_media_result(&media_name) {
                    let confirm = Confirm::new()
                        .with_prompt(format!(
                            "Is this the correct title: {} ({}), {}",
                            media_response.title, media_response.year, media_response.actors
                        ))
                        .interact()
                        .unwrap();

                    if confirm {
                        println!(
                            "Media confirmed: {} ({})",
                            media_response.title, media_response.year
                        );
                        break;
                    } else {
                        media_name = Input::new()
                            .with_prompt("Enter the IMDB ID of the media")
                            .interact_text()
                            .unwrap();
                    }
                } else {
                    println!("Media not found.");
                    break;
                }
            }

            // println!("Adding a movie... {}", media_name);
            // Call the function to add a movie
        }
        Actions::Mod => {
            println!("Modifying a movie...");
        }
        Actions::Del => {
            println!("Deleting a movie...");
        }
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
            "Is this the correct title: {} ({}), {}",
            media_response.title, media_response.year, media_response.actors
        ))
        .interact()
        .unwrap();

    confirm
}
