from datetime import datetime, timezone
import secrets
from typing import Any, Dict, List, Optional

from app.services.expert_auth_service import normalize_expert_id
from app.services.sqlite_store import get_conn

ALLOWED_CHILD_ICON_KEYS = (
    "pig",
    "fox",
    "owl",
    "cat",
    "bear",
    "alligator",
    "rabbit",
    "lion",
    "penguin",
)

MAX_CHILD_ID_RETRIES = 60


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def normalize_child_id(child_id: str) -> str:
    return (child_id or "").strip()


def normalize_name(value: str) -> str:
    return " ".join((value or "").strip().split())


def normalize_icon_key(icon_key: str) -> str:
    return (icon_key or "").strip().lower()


def _row_to_child(row: Any) -> Dict[str, Any]:
    payload: Dict[str, Any] = {
        "child_id": row["child_id"],
        "expert_id": row["expert_id"],
        "first_name": row["first_name"],
        "last_name": row["last_name"],
        "icon_key": row["icon_key"],
        "is_active": bool(row["is_active"]),
        "created_at": row["created_at"],
        "updated_at": row["updated_at"],
    }
    if "expert_name" in row.keys():
        payload["expert_name"] = row["expert_name"]
    return payload


def _ensure_expert_exists(expert_id: str) -> None:
    with get_conn() as conn:
        row = conn.execute(
            "SELECT 1 FROM experts WHERE expert_id = ?",
            (expert_id,),
        ).fetchone()
    if not row:
        raise ValueError("expert_id not found")


def _child_id_exists(child_id: str) -> bool:
    with get_conn() as conn:
        row = conn.execute(
            "SELECT 1 FROM children WHERE child_id = ?",
            (child_id,),
        ).fetchone()
    return row is not None


def generate_child_id() -> str:
    for _ in range(MAX_CHILD_ID_RETRIES):
        candidate = f"{secrets.randbelow(1_000_000):06d}"
        if not _child_id_exists(candidate):
            return candidate
    raise RuntimeError("child_id_generation_failed")


def get_child(child_id: str, include_inactive: bool = True) -> Optional[Dict[str, Any]]:
    child_id = normalize_child_id(child_id)
    if not child_id:
        return None

    query = """
        SELECT c.child_id, c.expert_id, c.first_name, c.last_name, c.icon_key,
               c.is_active, c.created_at, c.updated_at, e.display_name AS expert_name
        FROM children c
        LEFT JOIN experts e ON e.expert_id = c.expert_id
        WHERE c.child_id = ?
    """
    params: List[Any] = [child_id]
    if not include_inactive:
        query += " AND c.is_active = 1"

    with get_conn() as conn:
        row = conn.execute(query, tuple(params)).fetchone()

    if not row:
        return None
    return _row_to_child(row)


def list_children(
    expert_id: Optional[str] = None,
    include_inactive: bool = False,
) -> List[Dict[str, Any]]:
    filters: List[str] = []
    values: List[Any] = []

    normalized_expert_id = None
    if expert_id is not None:
        normalized_expert_id = normalize_expert_id(expert_id)
        if not normalized_expert_id:
            return []
        filters.append("lower(trim(c.expert_id)) = ?")
        values.append(normalized_expert_id)

    if not include_inactive:
        filters.append("c.is_active = 1")

    where_sql = ""
    if filters:
        where_sql = "WHERE " + " AND ".join(filters)

    with get_conn() as conn:
        rows = conn.execute(
            f"""
            SELECT c.child_id, c.expert_id, c.first_name, c.last_name, c.icon_key,
                   c.is_active, c.created_at, c.updated_at, e.display_name AS expert_name
            FROM children c
            LEFT JOIN experts e ON e.expert_id = c.expert_id
            {where_sql}
            ORDER BY lower(COALESCE(c.expert_id, '')), lower(c.last_name), lower(c.first_name), c.child_id
            """,
            tuple(values),
        ).fetchall()

    return [_row_to_child(row) for row in rows]


def create_child(
    expert_id: str,
    first_name: str,
    last_name: str,
    icon_key: str,
) -> Dict[str, Any]:
    expert_id = normalize_expert_id(expert_id)
    first_name = normalize_name(first_name)
    last_name = normalize_name(last_name)
    icon_key = normalize_icon_key(icon_key)

    if not expert_id:
        raise ValueError("expert_id is required")
    if not first_name:
        raise ValueError("first_name is required")
    if not last_name:
        raise ValueError("last_name is required")
    if icon_key not in ALLOWED_CHILD_ICON_KEYS:
        raise ValueError("icon_key is invalid")

    _ensure_expert_exists(expert_id)

    now = utc_now_iso()
    child_id = generate_child_id()

    try:
        with get_conn() as conn:
            conn.execute(
                """
                INSERT INTO children (
                    child_id, expert_id, first_name, last_name, icon_key, is_active, created_at, updated_at
                )
                VALUES (?, ?, ?, ?, ?, 1, ?, ?)
                """,
                (child_id, expert_id, first_name, last_name, icon_key, now, now),
            )
            conn.commit()
    except Exception as exc:
        message = str(exc)
        if "idx_children_unique_profile_per_expert" in message:
            raise RuntimeError("duplicate_child_profile") from exc
        if "UNIQUE constraint failed: children.child_id" in message:
            raise RuntimeError("duplicate_child_id") from exc
        raise

    created = get_child(child_id)
    if not created:
        raise RuntimeError("create_failed")
    return created


def update_child(
    child_id: str,
    expert_id: Optional[str] = None,
    first_name: Optional[str] = None,
    last_name: Optional[str] = None,
    icon_key: Optional[str] = None,
    is_active: Optional[bool] = None,
) -> Optional[Dict[str, Any]]:
    child_id = normalize_child_id(child_id)
    if not child_id:
        return None

    updates: List[str] = []
    values: List[Any] = []

    if expert_id is not None:
        normalized_expert_id = normalize_expert_id(expert_id)
        if normalized_expert_id:
            _ensure_expert_exists(normalized_expert_id)
            updates.append("expert_id = ?")
            values.append(normalized_expert_id)
        else:
            # empty value means unlink child from expert
            updates.append("expert_id = ?")
            values.append(None)

    if first_name is not None:
        cleaned_first = normalize_name(first_name)
        if not cleaned_first:
            raise ValueError("first_name cannot be empty")
        updates.append("first_name = ?")
        values.append(cleaned_first)

    if last_name is not None:
        cleaned_last = normalize_name(last_name)
        if not cleaned_last:
            raise ValueError("last_name cannot be empty")
        updates.append("last_name = ?")
        values.append(cleaned_last)

    if icon_key is not None:
        cleaned_icon = normalize_icon_key(icon_key)
        if cleaned_icon not in ALLOWED_CHILD_ICON_KEYS:
            raise ValueError("icon_key is invalid")
        updates.append("icon_key = ?")
        values.append(cleaned_icon)

    if is_active is not None:
        if not isinstance(is_active, bool):
            raise ValueError("is_active must be true or false")
        updates.append("is_active = ?")
        values.append(1 if is_active else 0)

    if not updates:
        return get_child(child_id)

    updates.append("updated_at = ?")
    values.append(utc_now_iso())
    values.append(child_id)

    try:
        with get_conn() as conn:
            cur = conn.execute(
                f"UPDATE children SET {', '.join(updates)} WHERE child_id = ?",
                tuple(values),
            )
            conn.commit()
    except Exception as exc:
        message = str(exc)
        if "idx_children_unique_profile_per_expert" in message:
            raise RuntimeError("duplicate_child_profile") from exc
        raise

    if cur.rowcount == 0:
        return None
    return get_child(child_id)


def deactivate_child(child_id: str) -> Optional[Dict[str, Any]]:
    return update_child(child_id, is_active=False)
