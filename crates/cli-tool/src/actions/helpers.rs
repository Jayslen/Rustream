use crate::types::{AssetType, BodyResponse, MediaAsset};
use dialoguer::{Confirm, Input};
use std::io;
use std::{fs, path::PathBuf};
use uuid::Uuid;

pub fn get_path_value(path: &Option<PathBuf>, path_msg: &str) -> PathBuf {
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

pub fn get_media_result(title: &String) -> Result<BodyResponse, Box<dyn std::error::Error>> {
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

pub fn confirm_media(media_response: &BodyResponse) -> bool {
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

// Prepare assets for copying to the destination directories
// with its uuid to add it then in db
pub fn prepare_assets(
    assets_dir: [&PathBuf; 2],
    media_name: &String,
) -> io::Result<Vec<MediaAsset>> {
    let mut assets = Vec::new();

    for path in assets_dir {
        for (index, entry) in fs::read_dir(path)?.enumerate() {
            let entry = entry?;
            let source_path = entry.path();
            let asset_type = if path.ends_with("posters") {
                AssetType::Poster
            } else {
                AssetType::Cover
            };

            let asset_type_str = match asset_type {
                AssetType::Poster => "poster",
                AssetType::Cover => "cover",
            };

            // Ignore directories
            if !source_path.is_file() {
                continue;
            }

            let extension = source_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("webp");

            let asset_id = Uuid::new_v4();

            // Example: la-la-land-2016-a81f29c4-poster-01.jpg
            let filename = format!("{media_name}-{asset_type_str}-{:02}.{extension}", index + 1);

            assets.push(MediaAsset {
                id: asset_id,
                filename,
                source_path,
                asset_type: match asset_type {
                    AssetType::Poster => AssetType::Poster,
                    AssetType::Cover => AssetType::Cover,
                },
            });
        }
    }

    Ok(assets)
}
