use crate::types::{BodyResponse, MediaAsset, MediaEntry, MediaType};
use rusqlite::{Connection, Transaction, params};
use std::{env, error::Error, fs, path::Path};
use uuid::Uuid;

const DEFAULT_DATABASE_PATH: &str = "/mnt/external-drive/streaming-service/streaming_service.db";

/// Add a movie and all related records in one transaction.
pub fn add_media(
    media: &MediaEntry,
    details: &BodyResponse,
    assets: &[MediaAsset],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if !matches!(media.media_type, MediaType::Movie) {
        return Err("only movies are supported by the database add operation".into());
    }

    let database_path = env::var("DATABASE_PATH")
        .or_else(|_| env::var("DATABASE_URL"))
        .unwrap_or_else(|_| DEFAULT_DATABASE_PATH.to_owned());
    if let Some(parent) = Path::new(&database_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut connection = Connection::open(database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    connection.execute_batch(include_str!("../../server/db/streaming_service_schema.sql"))?;
    add_movie(&mut connection, media, details, assets)
}

fn add_movie(
    connection: &mut Connection,
    media: &MediaEntry,
    details: &BodyResponse,
    assets: &[MediaAsset],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let transaction = connection.transaction()?;
    let title_id = media.uuid.to_string();
    let media_file_id = Uuid::new_v4().to_string();
    let release_year = parse_year(&details.year);
    let media_path = media.destination_path.to_string_lossy().into_owned();
    let file_name = media
        .destination_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let file_size = fs::metadata(&media.destination_path)?.len();
    let container = media
        .destination_path
        .extension()
        .and_then(|extension| extension.to_str());

    transaction.execute(
        "INSERT INTO titles (id, type, title, plot, release_year) VALUES (?1, 'movie', ?2, ?3, ?4)",
        params![title_id, details.title, details.plot, release_year],
    )?;
    transaction.execute(
        "INSERT INTO media_files (id, path, file_name, file_size, container) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![media_file_id, media_path, file_name, file_size as i64, container],
    )?;
    transaction.execute(
        "INSERT INTO movies (title_id, media_file_id) VALUES (?1, ?2)",
        params![title_id, media_file_id],
    )?;

    // Future implementation
    fn add_title() {
        // If title exists return the uuid. This is for convenience when adding a series
    }

    insert_genres(&transaction, &title_id, &details.genre)?;
    insert_credits(&transaction, &title_id, "director", &details.director)?;
    insert_credits(&transaction, &title_id, "writer", &details.writer)?;
    insert_credits(&transaction, &title_id, "actor", &details.actors)?;
    insert_assets(&transaction, &title_id, assets)?;
    transaction.commit()?;
    Ok(())
}

fn parse_year(year: &str) -> Option<i64> {
    year.split('–')
        .next()
        .and_then(|value| value.trim().parse().ok())
}

fn insert_genres(
    transaction: &Transaction<'_>,
    title_id: &str,
    genres: &str,
) -> rusqlite::Result<()> {
    for (position, genre) in comma_values(genres).enumerate() {
        let genre_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO genres (id, name) VALUES (?1, ?2) ON CONFLICT(name) DO NOTHING",
            params![genre_id, genre],
        )?;
        let existing_id: String = transaction.query_row(
            "SELECT id FROM genres WHERE name = ?1",
            params![genre],
            |row| row.get(0),
        )?;
        transaction.execute(
            "INSERT INTO title_genres (title_id, genre_id, position) VALUES (?1, ?2, ?3)",
            params![title_id, existing_id, position as i64],
        )?;
    }
    Ok(())
}

fn insert_credits(
    transaction: &Transaction<'_>,
    title_id: &str,
    credit_type: &str,
    names: &str,
) -> rusqlite::Result<()> {
    for (position, name) in comma_values(names).enumerate() {
        let person_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO people (id, name) VALUES (?1, ?2) ON CONFLICT(name) DO NOTHING",
            params![person_id, name],
        )?;
        let existing_id: String = transaction.query_row(
            "SELECT id FROM people WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )?;
        transaction.execute(
            "INSERT INTO title_credits (title_id, person_id, credit_type, position) VALUES (?1, ?2, ?3, ?4)",
            params![title_id, existing_id, credit_type, position as i64],
        )?;
    }
    Ok(())
}

fn insert_assets(
    transaction: &Transaction<'_>,
    title_id: &str,
    assets: &[MediaAsset],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut positions = [0_i64; 2];
    let mut primaries = [false; 2];
    for asset in assets {
        let image_type = asset.asset_type.as_str();
        let type_index = if image_type == "poster" { 0 } else { 1 };
        let image_id = asset.id.to_string();
        transaction.execute(
            "INSERT INTO images (id, path, mime_type) VALUES (?1, ?2, ?3)",
            params![
                image_id,
                asset.source_path.to_string_lossy(),
                mime_type(&asset.source_path)
            ],
        )?;
        transaction.execute(
            "INSERT INTO title_images (id, title_id, image_id, image_type, is_primary, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![Uuid::new_v4().to_string(), title_id, image_id, image_type,
                (!primaries[type_index]) as i64, positions[type_index]],
        )?;
        positions[type_index] += 1;
        primaries[type_index] = true;
    }
    Ok(())
}

fn comma_values(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "N/A")
}

fn mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}
