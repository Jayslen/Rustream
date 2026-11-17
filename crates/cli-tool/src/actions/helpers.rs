use crate::types::BodyResponse;
use dialoguer::{Confirm, Input};
use std::{fs, path::PathBuf};

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
