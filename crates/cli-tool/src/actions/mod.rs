use crate::actions::helpers::prepare_assets;
use crate::types::{BodyResponse, Cli, MediaAsset, MediaEntry};
use crate::{database, filesystem};
use dialoguer::Input;

mod helpers;

pub const STORAGE_PATH: &str = "/mnt/external-drive/streaming-service";

pub fn add(cli: &Cli) {
    let mut media_name = if let Some(media_id) = &cli.media_id {
        media_id.clone()
    } else {
        Input::new()
            .with_prompt("Enter the IMDB ID of the media \n")
            .interact_text()
            .unwrap()
    };

    let mut selected_media: Option<BodyResponse> = None;

    while match helpers::get_media_result(&media_name) {
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

    let media_details = selected_media.expect("Media details should be available at this point");

    let assets: Vec<MediaAsset>;

    // Get assets ready with path and filename formated to be stored in the assets directory
    // scope it to drop the variables after this block
    {
        let covers_dir = helpers::get_path_value(&cli.cover_img_dir, "covers directory");
        let posters_dir = helpers::get_path_value(&cli.poster_img_dir, "posters directory");

        assets = prepare_assets(
            [&posters_dir, &covers_dir],
            &media_details.title.replace(' ', "-").to_lowercase(),
        )
        .unwrap();
    }

    let media_path = helpers::get_path_value(&cli.media_path, "media file");

    // Prepare the media entry basic information to be used in the filesystem and database operations
    let media_entry = MediaEntry::new(cli.media_type, media_path, &media_details.title);

    filesystem::add_media(&media_entry, &assets)
        .expect("Could not copy media and assets to the destination directories");

    database::add_media(&media_entry, &media_details, &assets)
        .expect("Could not add media metadata to the database");

    // println!("Media added successfully: {:?}", assets);
}
