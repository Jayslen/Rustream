use crate::types::{AssetType, MediaAsset, MediaEntry};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

const MEDIA_PATH: &str = "/mnt/external-drive/streaming-service";

pub fn add_media(
    media: &MediaEntry,
    media_assets: &[MediaAsset],
) -> Result<(), Box<dyn std::error::Error>> {
    // SEGMENT Series and Movies
    let root = Path::new(MEDIA_PATH);

    let movie_dir = root.join("media/movies");
    let assets_dir = root.join("assets/movies").join(&media.file_name);

    let movie_source = media.source_path.clone();

    let movie_destination = movie_dir.join(format!(
        "{}.{}",
        media.file_name,
        movie_source
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("mkv")
    ));

    let poster_destination = assets_dir.join("posters");
    let cover_destination = assets_dir.join("covers");

    thread::scope(|s| {
        let assets_thread = s.spawn(|| {
            copy_assets(media_assets, &poster_destination, &cover_destination)?;
            println!(
                "Copied assets to {} and {}",
                poster_destination.display(),
                cover_destination.display()
            );

            Ok::<(), std::io::Error>(())
        });

        let movie_thread = s.spawn(|| -> std::io::Result<()> {
            fs::create_dir_all(&movie_dir)?;
            fs::copy(&movie_source, &movie_destination)?;
            println!(
                "Copied movie from {} to {}",
                movie_source.display(),
                movie_destination.display()
            );
            Ok(())
        });

        movie_thread.join().map_err(|_| "Movie thread panicked")??;

        assets_thread
            .join()
            .map_err(|_| "Assets thread panicked")??;

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

fn copy_assets(
    assets: &[MediaAsset],
    cover_destination: &PathBuf,
    poster_destination: &PathBuf,
) -> std::io::Result<()> {
    for asset in assets {
        let destination = match asset.asset_type {
            AssetType::Poster => poster_destination.join(&asset.filename),
            AssetType::Cover => cover_destination.join(&asset.filename),
        };

        fs::create_dir_all(destination.parent().unwrap())?;
        fs::copy(&asset.source_path, &destination)?;
    }

    Ok(())
}
