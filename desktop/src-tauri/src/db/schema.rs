pub const CREATE_TABLES: &str = "
CREATE TABLE IF NOT EXISTS users (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT    NOT NULL,
    role    TEXT    NOT NULL CHECK(role IN ('parent', 'kid'))
);

CREATE TABLE IF NOT EXISTS voice_embeddings (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id   INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    embedding BLOB    NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS kid_tags (
    kid_id INTEGER NOT NULL REFERENCES users(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (kid_id, tag_id)
);

CREATE TABLE IF NOT EXISTS videos (
    id               TEXT PRIMARY KEY,
    title            TEXT,
    thumbnail_url    TEXT,
    duration_seconds INTEGER,
    local_video_path TEXT
);

CREATE TABLE IF NOT EXISTS video_tags (
    video_id TEXT    NOT NULL REFERENCES videos(id),
    tag_id   INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (video_id, tag_id)
);

CREATE TABLE IF NOT EXISTS video_assignments (
    kid_id   INTEGER NOT NULL REFERENCES users(id),
    video_id TEXT    NOT NULL REFERENCES videos(id),
    answers  TEXT,
    PRIMARY KEY (kid_id, video_id)
);

CREATE TABLE IF NOT EXISTS segments (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id      TEXT    NOT NULL REFERENCES videos(id),
    start_seconds INTEGER NOT NULL,
    end_seconds   INTEGER NOT NULL,
    best_question TEXT
);

CREATE TABLE IF NOT EXISTS questions (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    segment_id                  INTEGER NOT NULL REFERENCES segments(id),
    qtype                       TEXT    NOT NULL,
    question                    TEXT    NOT NULL,
    answer                      TEXT    NOT NULL,
    followup_correct_question   TEXT,
    followup_correct_answer     TEXT,
    followup_wrong_question     TEXT,
    followup_wrong_answer       TEXT,
    rank                        INTEGER
);

CREATE TABLE IF NOT EXISTS answers (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    kid_id           INTEGER NOT NULL REFERENCES users(id),
    video_id         TEXT    NOT NULL REFERENCES videos(id),
    segment_id       INTEGER NOT NULL REFERENCES segments(id),
    transcript       TEXT    NOT NULL,
    is_correct       INTEGER NOT NULL,
    similarity_score REAL    NOT NULL,
    mood             TEXT
);

CREATE TABLE IF NOT EXISTS frames (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id            TEXT    NOT NULL REFERENCES videos(id),
    frame_number        INTEGER NOT NULL,
    timestamp_seconds   INTEGER NOT NULL,
    timestamp_formatted TEXT    NOT NULL,
    filename            TEXT    NOT NULL,
    file_path           TEXT    NOT NULL,
    is_keyframe         INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS app_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";
