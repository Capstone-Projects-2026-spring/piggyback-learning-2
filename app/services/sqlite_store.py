import sqlite3
from pathlib import Path

from app.settings import SQLITE_PATH

def get_conn() -> sqlite3.Connection:
    db_path = Path(SQLITE_PATH)
    db_path.parent.mkdir(parents=True, exist_ok=True)

    conn = sqlite3.connect(db_path, check_same_thread=False)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON;")
    return conn


def init_db() -> None:
    with get_conn() as conn:
        conn.executescript(
            """
            CREATE TABLE IF NOT EXISTS experts (
                expert_id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS video_assignments (
                video_id TEXT PRIMARY KEY,
                expert_id TEXT NULL,
                assignment_source TEXT NOT NULL CHECK (assignment_source IN ('admin', 'expert_claim', 'unassigned')),
                assigned_at TEXT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (expert_id) REFERENCES experts(expert_id)
                    ON UPDATE CASCADE
                    ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_video_assignments_expert_id
                ON video_assignments (expert_id);
            """
        )
        conn.commit()