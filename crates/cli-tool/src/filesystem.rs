use super::MediaEntry;
use crate::BodyResponse;
use std::io;
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};
use uuid::Uuid;

const MEDIA_PATH: &str = "/mnt/external-drive/streaming-service";

#[derive(Debug)]
pub struct MediaAsset {
    pub id: Uuid,
    pub path: PathBuf,
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

pub fn add_media(
    media: &MediaEntry,
    media_details: &BodyResponse,
) -> Result<Vec<MediaAsset>, Box<dyn std::error::Error + Send + Sync>> {
    let uuid_prefix = media.uuid.simple().to_string()[..8].to_string();

    let slug = media_details.title.replace(' ', "-").to_lowercase();

    let media_name = format!("{slug}-{}-{uuid_prefix}", media_details.year);

    let poster_media_name = media_name.clone();
    let cover_media_name = media_name.clone();

    let root = Path::new(MEDIA_PATH);

    let movie_dir = root.join("media/movies");
    let assets_dir = root.join("assets/movies").join(&media_name);

    let movie_source = media.media_path.clone();
    let poster_source = media.poster_path.clone();
    let cover_source = media.cover_path.clone();

    let movie_destination = movie_dir.join(format!(
        "{}.{}",
        media_name,
        movie_source
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("mkv")
    ));

    let poster_destination = assets_dir.join("posters");
    let cover_destination = assets_dir.join("covers");

    // Movie thread
    let movie_thread = thread::spawn(move || -> std::io::Result<()> {
        fs::create_dir_all(&movie_dir)?;

        fs::copy(movie_source, movie_destination)?;

        Ok(())
    });

    // Posters thread
    let poster_thread = thread::spawn(move || {
        copy_assets(
            &poster_source,
            &poster_destination,
            &poster_media_name,
            AssetType::Poster,
        )
    });

    // Covers thread
    let cover_thread = thread::spawn(move || {
        copy_assets(
            &cover_source,
            &cover_destination,
            &cover_media_name,
            AssetType::Cover,
        )
    });

    movie_thread.join().map_err(|_| "Movie thread panicked")??;

    let mut assets = poster_thread
        .join()
        .map_err(|_| "Poster thread panicked")??;

    assets.extend(cover_thread.join().map_err(|_| "Cover thread panicked")??);

    Ok(assets)
}

fn copy_assets(
    source: &Path,
    destination: &Path,
    media_name: &str,
    asset_type: AssetType,
) -> io::Result<Vec<MediaAsset>> {
    fs::create_dir_all(destination)?;

    let mut assets = Vec::new();
    let asset_type_str = match asset_type {
        AssetType::Poster => "poster",
        AssetType::Cover => "cover",
    };

    for (index, entry) in fs::read_dir(source)?.enumerate() {
        let entry = entry?;
        let source_path = entry.path();

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

        let destination_path = destination.join(filename);

        fs::copy(&source_path, &destination_path)?;

        assets.push(MediaAsset {
            id: asset_id,
            path: destination_path,
            asset_type: match asset_type {
                AssetType::Poster => AssetType::Poster,
                AssetType::Cover => AssetType::Cover,
            },
        });
    }

    Ok(assets)
}
