
#password hasing + safe comparison + random
import hashlib
import hmac
import os
import re
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

from app.services.sqlite_store import get_conn

#label store in a hash string
PBKDF2_ALGO = "pbkdf2_sha256"
PBKDF2_ITERATIONS = 120_000
#rnadom salt by 16 bytes
SALT_BYTES = 16
EXPERT_ID_PATTERN = re.compile(r"^[a-z0-9][a-z0-9_-]{2,49}$")


def utc_now_iso() -> str:
    # UTC ISO timestamps keep storage simple and sortable.
    return datetime.now(timezone.utc).isoformat()

#never store plain text passwsowrds
def hash_password(password: str) -> str:
    if not isinstance(password, str) or not password:
        raise ValueError("Password is required")

    salt = os.urandom(SALT_BYTES)
    digest = hashlib.pbkdf2_hmac(
        "sha256",
        password.encode("utf-8"),
        salt,
        PBKDF2_ITERATIONS,
    )
    return f"{PBKDF2_ALGO}${PBKDF2_ITERATIONS}${salt.hex()}${digest.hex()}"


def verify_password(password: str, stored_hash: str) -> bool:
    try:
        algo, iteration_text, salt_hex, digest_hex = stored_hash.split("$", 3)
        if algo != PBKDF2_ALGO:
            return False

        iterations = int(iteration_text)
        salt = bytes.fromhex(salt_hex)
        expected_digest = bytes.fromhex(digest_hex)
    except Exception:
        return False

    candidate = hashlib.pbkdf2_hmac(
        "sha256",
        password.encode("utf-8"),
        salt,
        iterations,
    )
    return hmac.compare_digest(candidate, expected_digest)


def normalize_expert_id(expert_id: str) -> str:
    return (expert_id or "").strip().lower()


def is_valid_expert_id(expert_id: str) -> bool:
    return bool(EXPERT_ID_PATTERN.fullmatch(normalize_expert_id(expert_id)))


def _row_to_expert(row: Any) -> Dict[str, Any]:
    return {
        "expert_id": row["expert_id"],
        "display_name": row["display_name"],
        "is_active": bool(row["is_active"]),
        "created_at": row["created_at"],
        "updated_at": row["updated_at"],
    }


def list_experts() -> List[Dict[str, Any]]:
    with get_conn() as conn:
        rows = conn.execute(
            """
            SELECT expert_id, display_name, is_active, created_at, updated_at
            FROM experts
            ORDER BY expert_id ASC
            """
        ).fetchall()
    return [_row_to_expert(row) for row in rows]


def get_expert(expert_id: str) -> Optional[Dict[str, Any]]:
    expert_id = normalize_expert_id(expert_id)
    if not expert_id:
        return None

    with get_conn() as conn:
        row = conn.execute(
            """
            SELECT expert_id, display_name, is_active, created_at, updated_at
            FROM experts
            WHERE expert_id = ?
            """,
            (expert_id,),
        ).fetchone()

    if not row:
        return None
    return _row_to_expert(row)


def create_expert(expert_id: str, display_name: str, password: str) -> Dict[str, Any]:
    expert_id = normalize_expert_id(expert_id)
    display_name = (display_name or "").strip()

    if not is_valid_expert_id(expert_id):
        raise ValueError("expert_id must be 3-50 chars: lowercase letters, numbers, _ or -")
    if not display_name:
        raise ValueError("display_name is required")
    if not password:
        raise ValueError("password is required")

    now = utc_now_iso()
    password_hash = hash_password(password)

    try:
        with get_conn() as conn:
            conn.execute(
                """
                INSERT INTO experts (expert_id, display_name, password_hash, is_active, created_at, updated_at)
                VALUES (?, ?, ?, 1, ?, ?)
                """,
                (expert_id, display_name, password_hash, now, now),
            )
            conn.commit()
    except Exception as exc:
        if "UNIQUE constraint failed" in str(exc):
            raise RuntimeError("duplicate_expert_id") from exc
        raise

    created = get_expert(expert_id)
    if not created:
        raise RuntimeError("create_failed")
    return created


def update_expert(
    expert_id: str,
    display_name: Optional[str] = None,
    password: Optional[str] = None,
    is_active: Optional[bool] = None,
) -> Optional[Dict[str, Any]]:
    expert_id = normalize_expert_id(expert_id)
    if not expert_id:
        return None

    updates = []
    values: List[Any] = []

    if display_name is not None:
        cleaned = display_name.strip()
        if not cleaned:
            raise ValueError("display_name cannot be empty")
        updates.append("display_name = ?")
        values.append(cleaned)

    if password is not None:
        if not password:
            raise ValueError("password cannot be empty")
        updates.append("password_hash = ?")
        values.append(hash_password(password))

    if is_active is not None:
        updates.append("is_active = ?")
        values.append(1 if is_active else 0)

    if not updates:
        return get_expert(expert_id)

    updates.append("updated_at = ?")
    values.append(utc_now_iso())
    values.append(expert_id)

    with get_conn() as conn:
        cur = conn.execute(
            f"UPDATE experts SET {', '.join(updates)} WHERE expert_id = ?",
            tuple(values),
        )
        conn.commit()

    if cur.rowcount == 0:
        return None
    return get_expert(expert_id)


def deactivate_expert(expert_id: str) -> Optional[Dict[str, Any]]:
    return update_expert(expert_id, is_active=False)

def delete_expert(expert_id: str) -> bool:
    expert_id = normalize_expert_id(expert_id)
    with get_conn() as conn:
        cur = conn.execute("DELETE FROM experts WHERE expert_id = ?", (expert_id,))
        conn.commit()
        return cur.rowcount > 0


def authenticate_expert(expert_id: str, password: str) -> Optional[Dict[str, Any]]:
    expert_id = normalize_expert_id(expert_id)
    if not expert_id or not password:
        return None

    with get_conn() as conn:
        row = conn.execute(
            """
            SELECT expert_id, display_name, password_hash, is_active
            FROM experts
            WHERE expert_id = ?
            """,
            (expert_id,),
        ).fetchone()

    if not row:
        return None
    if not bool(row["is_active"]):
        return None
    if not verify_password(password, row["password_hash"]):
        return None

    return {
        "expert_id": row["expert_id"],
        "display_name": row["display_name"],
    }


def ensure_video_assignment_rows(video_ids: List[str]) -> None:
    # Keeps assignment table in sync with downloads folders.
    now = utc_now_iso()
    normalized = sorted({(video_id or "").strip() for video_id in video_ids if (video_id or "").strip()})
    if not normalized:
        return

    with get_conn() as conn:
        for video_id in normalized:
            conn.execute(
                """
                INSERT OR IGNORE INTO video_assignments (
                    video_id, expert_id, assignment_source, assigned_at, updated_at
                ) VALUES (?, NULL, 'unassigned', NULL, ?)
                """,
                (video_id, now),
            )
        conn.commit()

def add_video_assignment(video_id,expert_id,source : str = "admin"):
    # Upsert a video-expert pair into the many-to-many table.
    video_id = (video_id or "").strip()
    expert_id = normalize_expert_id(expert_id)
    
    if not video_id:
        raise ValueError ("video_id is required")
    
    if not expert_id:
        raise ValueError("expert_id is required")
    
    if source not in {"admin", "expert_claim", "unassigned"}:
        raise ValueError("invalid assignment source")
    # If the pair already exists, just update the source and timestamp.
    now = utc_now_iso()
    with get_conn() as conn:
        conn.execute("""
        INSERT INTO video_expert_assignments (video_id, expert_id, assignment_source, assigned_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(video_id, expert_id) DO UPDATE SET
        assignment_source = excluded.assignment_source,
        updated_at = excluded.updated_at """,(video_id,expert_id,source,now,now))
        conn.commit()

def remove_video_assignment(video_id, expert_id):
    video_id = (video_id or "").strip()
    expert_id = (expert_id or "").strip()
    
    if not video_id:
        raise ValueError ("video_id is required")
    
    if not expert_id:
        raise ValueError("expert_id is required")
    
    with get_conn() as conn:
        conn.execute(
            "DELETE FROM video_expert_assignments WHERE video_id = ? AND expert_id = ?",
        (video_id, expert_id)
        )

        conn.commit()

def list_experts_for_video(video_id:str):

    video_id = (video_id or "").strip()
    if not video_id:
        raise ValueError("video_id is required")

    with get_conn() as conn:
        rows = conn.execute("""
            SELECT vea.*, e.display_name AS expert_name
            FROM video_expert_assignments vea
            JOIN experts e ON e.expert_id = vea.expert_id
            WHERE vea.video_id = ?
        """, (video_id,)).fetchall()   
    return [dict(row) for row in rows]

def list_video_assignments() -> List[Dict[str, Any]]:
    # Returns all video-expert pairs — one row per pair, not one row per video.
    with get_conn() as conn:
        rows = conn.execute(
            """
            SELECT vea.video_id, vea.expert_id, vea.assignment_source, vea.assigned_at, vea.updated_at,
                   e.display_name AS expert_name
            FROM video_expert_assignments vea
            JOIN experts e ON e.expert_id = vea.expert_id
            ORDER BY vea.video_id ASC
            """
        ).fetchall()

    return [dict(row) for row in rows]

def list_video_ids_for_expert(expert_id: str) -> List[str]:
    expert_id = normalize_expert_id(expert_id)
    if not expert_id:
        return []

    with get_conn() as conn:
        rows = conn.execute(
            """
            SELECT DISTINCT video_id
            FROM video_expert_assignments
            WHERE lower(trim(expert_id)) = ?
            ORDER BY video_id ASC
            """,
            (expert_id,),
        ).fetchall()

    return [str(row["video_id"]) for row in rows]

def can_expert_access_video(expert_id: str, video_id: str) -> bool:
    # Pair-based check — True if this expert has any assignment row for this video.
    expert_id = normalize_expert_id(expert_id)
    video_id = (video_id or "").strip().lower()
    if not expert_id or not video_id:
        return False

    with get_conn() as conn:
        row = conn.execute(
            """
            SELECT 1
            FROM video_expert_assignments
            WHERE lower(trim(video_id)) = ? AND lower(trim(expert_id)) = ?
            """,
            (video_id, expert_id),
        ).fetchone()

    return row is not None
    
def claim_video_for_expert(expert_id: str, video_id: str) -> None:
    # Assigned-only mode: confirm existing assignment, do not create new access.
    expert_id = normalize_expert_id(expert_id)
    normalized_video_id = (video_id or "").strip().lower()
    if not expert_id or not normalized_video_id:
        raise ValueError("expert_id and video_id are required")

    now = utc_now_iso()
    with get_conn() as conn:
        existing = conn.execute(
            """
            SELECT 1
            FROM video_expert_assignments
            WHERE lower(trim(video_id)) = ? AND lower(trim(expert_id)) = ?
            """,
            (normalized_video_id, expert_id),
        ).fetchone()
        if not existing:
            raise RuntimeError("assignment_not_found")

        conn.execute(
            """
            UPDATE video_expert_assignments
            SET updated_at = ?
            WHERE lower(trim(video_id)) = ? AND lower(trim(expert_id)) = ?
            """,
            (now, normalized_video_id, expert_id),
        )
        conn.commit()
