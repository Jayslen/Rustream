PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

-- =========================================================
-- IMAGES
-- Stores image metadata. Images can be reused by multiple titles.
-- =========================================================

CREATE TABLE IF NOT EXISTS images (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    mime_type TEXT NOT NULL,
    width INTEGER,
    height INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CHECK (width IS NULL OR width > 0),
    CHECK (height IS NULL OR height > 0)
);

-- =========================================================
-- TITLES
-- Common metadata for movies and series.
-- =========================================================

CREATE TABLE IF NOT EXISTS titles (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL CHECK (type IN ('movie', 'series')),
    title TEXT NOT NULL,
    original_title TEXT,
    description TEXT,
    release_year INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CHECK (
        release_year IS NULL
        OR release_year BETWEEN 1800 AND 2100
    )
);

-- =========================================================
-- TITLE IMAGES
-- Multiple posters and covers for each movie or series.
-- image_type: poster | cover
-- is_primary: only one primary image per type/title.
-- position: ordering in galleries.
-- =========================================================

CREATE TABLE IF NOT EXISTS title_images (
    id TEXT PRIMARY KEY,
    title_id TEXT NOT NULL,
    image_id TEXT NOT NULL,
    image_type TEXT NOT NULL CHECK (
        image_type IN ('poster', 'cover')
    ),
    is_primary INTEGER NOT NULL DEFAULT 0 CHECK (
        is_primary IN (0, 1)
    ),
    position INTEGER NOT NULL DEFAULT 0 CHECK (
        position >= 0
    ),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (title_id)
        REFERENCES titles(id)
        ON DELETE CASCADE,

    FOREIGN KEY (image_id)
        REFERENCES images(id)
        ON DELETE CASCADE,

    UNIQUE (title_id, image_id, image_type)
);

-- =========================================================
-- MEDIA FILES
-- Physical video files stored on disk.
-- =========================================================

CREATE TABLE IF NOT EXISTS media_files (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    container TEXT,
    video_codec TEXT,
    audio_codec TEXT,
    width INTEGER,
    height INTEGER,
    duration REAL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CHECK (file_size >= 0),
    CHECK (width IS NULL OR width > 0),
    CHECK (height IS NULL OR height > 0),
    CHECK (duration IS NULL OR duration >= 0)
);

-- =========================================================
-- MOVIES
-- A movie title references one media file.
-- =========================================================

CREATE TABLE IF NOT EXISTS movies (
    title_id TEXT PRIMARY KEY,
    media_file_id TEXT NOT NULL UNIQUE,

    FOREIGN KEY (title_id)
        REFERENCES titles(id)
        ON DELETE CASCADE,

    FOREIGN KEY (media_file_id)
        REFERENCES media_files(id)
        ON DELETE RESTRICT
);

-- =========================================================
-- SEASONS
-- A series can contain multiple seasons.
-- =========================================================

CREATE TABLE IF NOT EXISTS seasons (
    id TEXT PRIMARY KEY,
    series_id TEXT NOT NULL,
    season_number INTEGER NOT NULL,
    title TEXT,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (series_id)
        REFERENCES titles(id)
        ON DELETE CASCADE,

    UNIQUE (series_id, season_number),

    CHECK (season_number >= 0)
);

-- =========================================================
-- EPISODES
-- An episode references one media file and one season.
-- =========================================================

CREATE TABLE IF NOT EXISTS episodes (
    id TEXT PRIMARY KEY,
    series_id TEXT NOT NULL,
    season_id TEXT NOT NULL,
    media_file_id TEXT NOT NULL UNIQUE,
    episode_number INTEGER NOT NULL,
    title TEXT,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (series_id)
        REFERENCES titles(id)
        ON DELETE CASCADE,

    FOREIGN KEY (season_id)
        REFERENCES seasons(id)
        ON DELETE CASCADE,

    FOREIGN KEY (media_file_id)
        REFERENCES media_files(id)
        ON DELETE RESTRICT,

    UNIQUE (season_id, episode_number),

    CHECK (episode_number > 0)
);

-- =========================================================
-- INDEXES
-- =========================================================

CREATE INDEX IF NOT EXISTS idx_titles_type
    ON titles(type);

CREATE INDEX IF NOT EXISTS idx_titles_title
    ON titles(title);

CREATE INDEX IF NOT EXISTS idx_title_images_title
    ON title_images(title_id, image_type, position);

CREATE INDEX IF NOT EXISTS idx_title_images_image
    ON title_images(image_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_title_images_primary_poster
    ON title_images(title_id)
    WHERE image_type = 'poster' AND is_primary = 1;

CREATE UNIQUE INDEX IF NOT EXISTS idx_title_images_primary_cover
    ON title_images(title_id)
    WHERE image_type = 'cover' AND is_primary = 1;

CREATE INDEX IF NOT EXISTS idx_movies_media_file
    ON movies(media_file_id);

CREATE INDEX IF NOT EXISTS idx_seasons_series
    ON seasons(series_id);

CREATE INDEX IF NOT EXISTS idx_episodes_series
    ON episodes(series_id);

CREATE INDEX IF NOT EXISTS idx_episodes_season
    ON episodes(season_id);

CREATE INDEX IF NOT EXISTS idx_episodes_media_file
    ON episodes(media_file_id);

-- =========================================================
-- VALIDATION TRIGGERS
-- =========================================================

-- A movie must reference a title of type 'movie'.
CREATE TRIGGER IF NOT EXISTS validate_movie_title_insert
BEFORE INSERT ON movies
BEGIN
    SELECT CASE
        WHEN (
            SELECT type FROM titles WHERE id = NEW.title_id
        ) IS NOT 'movie'
        THEN RAISE(ABORT, 'Movie must reference a title of type movie')
    END;
END;

CREATE TRIGGER IF NOT EXISTS validate_movie_title_update
BEFORE UPDATE OF title_id ON movies
BEGIN
    SELECT CASE
        WHEN (
            SELECT type FROM titles WHERE id = NEW.title_id
        ) IS NOT 'movie'
        THEN RAISE(ABORT, 'Movie must reference a title of type movie')
    END;
END;

-- A season must reference a title of type 'series'.
CREATE TRIGGER IF NOT EXISTS validate_season_series_insert
BEFORE INSERT ON seasons
BEGIN
    SELECT CASE
        WHEN (
            SELECT type FROM titles WHERE id = NEW.series_id
        ) IS NOT 'series'
        THEN RAISE(ABORT, 'Season must reference a title of type series')
    END;
END;

CREATE TRIGGER IF NOT EXISTS validate_season_series_update
BEFORE UPDATE OF series_id ON seasons
BEGIN
    SELECT CASE
        WHEN (
            SELECT type FROM titles WHERE id = NEW.series_id
        ) IS NOT 'series'
        THEN RAISE(ABORT, 'Season must reference a title of type series')
    END;
END;

-- An episode must reference a series and a season belonging to that series.
CREATE TRIGGER IF NOT EXISTS validate_episode_series_insert
BEFORE INSERT ON episodes
BEGIN
    SELECT CASE
        WHEN (
            SELECT type FROM titles WHERE id = NEW.series_id
        ) IS NOT 'series'
        THEN RAISE(ABORT, 'Episode must reference a title of type series')
    END;

    SELECT CASE
        WHEN (
            SELECT series_id FROM seasons WHERE id = NEW.season_id
        ) IS NOT NEW.series_id
        THEN RAISE(ABORT, 'Episode season must belong to the referenced series')
    END;
END;

CREATE TRIGGER IF NOT EXISTS validate_episode_series_update
BEFORE UPDATE OF series_id, season_id ON episodes
BEGIN
    SELECT CASE
        WHEN (
            SELECT type FROM titles WHERE id = NEW.series_id
        ) IS NOT 'series'
        THEN RAISE(ABORT, 'Episode must reference a title of type series')
    END;

    SELECT CASE
        WHEN (
            SELECT series_id FROM seasons WHERE id = NEW.season_id
        ) IS NOT NEW.series_id
        THEN RAISE(ABORT, 'Episode season must belong to the referenced series')
    END;
END;

-- Prevent changing title type when it already has movie/season records.
CREATE TRIGGER IF NOT EXISTS prevent_title_type_change
BEFORE UPDATE OF type ON titles
WHEN OLD.type IS NOT NEW.type
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1 FROM movies WHERE title_id = OLD.id
        )
        OR EXISTS (
            SELECT 1 FROM seasons WHERE series_id = OLD.id
        )
        THEN RAISE(ABORT, 'Cannot change title type while related records exist')
    END;
END;

-- Automatically update titles.updated_at.
CREATE TRIGGER IF NOT EXISTS update_title_timestamp
AFTER UPDATE ON titles
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
    UPDATE titles
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

COMMIT;
