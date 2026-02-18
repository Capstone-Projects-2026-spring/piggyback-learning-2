# admin_routes.py
import json
import csv
from typing import Any, Dict, List, Optional

from fastapi import (
    APIRouter,
    Body,
    Form,
    HTTPException,
    Request,
    WebSocket,
    WebSocketDisconnect,
)
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates
#Pulls shared path from settings.py so all module uses the same directory
from app.settings import DOWNLOADS_DIR, TEMPLATES_DIR
# ----- Local paths (keep consistent with main.py) -----
#maybe needed if not delete later, should be duplicates with shared setting imports.
#TODO- might delete this 
# BASE_DIR = Path(__file__).parent.resolve()
# TEMPLATES_DIR = BASE_DIR / "templates"
# DOWNLOADS_DIR = BASE_DIR / "downloads"

# Use shared templates path from app.settings to avoid path drift across modules.
templates = Jinja2Templates(directory=str(TEMPLATES_DIR))

# Three routers:
#  - pages: mounted under /admin
#  - api:   mounted under /api
#  - ws:    mounted with NO prefix (keeps /ws/... as-is)
router_admin_pages = APIRouter()
router_admin_api = APIRouter()
router_admin_ws = APIRouter()

# ===== Helpers duplicated here (tiny / no circulars) =====
def format_hhmmss(total_seconds: int) -> str:
    h = total_seconds // 3600
    m = (total_seconds % 3600) // 60
    s = total_seconds % 60
    if h > 0:
        return f"{h:02d}:{m:02d}:{s:02d}"
    return f"{m:02d}:{s:02d}"


def _collect_downloaded_videos(include_without_frames: bool = False) -> List[Dict[str, Any]]:
    """
    Enumerate downloads/<video_id> folders so the admin UI can reuse existing assets.
    """
    entries: List[Dict[str, Any]] = []
    if not DOWNLOADS_DIR.exists():
        return entries

    for folder in sorted(DOWNLOADS_DIR.iterdir()):
        if not folder.is_dir():
            continue

        video_id = folder.name
        meta_path = folder / "meta.json"
        title = video_id
        duration_seconds: Optional[int] = None
        if meta_path.exists():
            try:
                meta = json.loads(meta_path.read_text(encoding="utf-8"))
                title = meta.get("title") or title
                duration_seconds = meta.get("duration")
            except Exception:
                pass

        frames_dir = folder / "extracted_frames"
        frames_json = frames_dir / "frame_data.json"
        has_frames = frames_json.exists()
        frame_count = None
        if frames_json.exists():
            try:
                frame_payload = json.loads(frames_json.read_text(encoding="utf-8"))
                frame_count = frame_payload.get("video_info", {}).get("extracted_frames")
            except Exception:
                frame_count = None

        questions_path = folder / "questions" / f"{video_id}.json"
        has_questions = questions_path.exists()
        if not include_without_frames and not has_frames:
            continue

        duration_label = None
        if duration_seconds:
            try:
                duration_label = format_hhmmss(int(float(duration_seconds)))
            except Exception:
                duration_label = None

        entry: Dict[str, Any] = {
            "video_id": video_id,
            "title": title,
            "duration_seconds": duration_seconds,
            "duration_formatted": duration_label,
            "has_frames": has_frames,
            "frame_count": frame_count,
            "has_questions": has_questions,
        }
        if has_frames:
            try:
                entry["frames_dir"] = (
                    f"/downloads/{frames_dir.relative_to(DOWNLOADS_DIR).as_posix()}"
                )
            except ValueError:
                entry["frames_dir"] = None
        if has_questions:
            try:
                entry["question_file"] = (
                    f"/downloads/{questions_path.relative_to(DOWNLOADS_DIR).as_posix()}"
                )
            except ValueError:
                entry["question_file"] = None
        entries.append(entry)

    return entries


def _segment_frame_debug(video_id: str, start: int, end: int) -> Dict[str, Any]:
    """
    Inspect extracted frame coverage for a segment to explain generation failures.
    """
    frames_dir = DOWNLOADS_DIR / video_id / "extracted_frames"
    csv_path = frames_dir / "frame_data.csv"
    debug: Dict[str, Any] = {"video_id": video_id, "start": start, "end": end}

    if not frames_dir.exists():
        debug["reason"] = "frames_dir_missing"
        return debug
    if not csv_path.exists():
        debug["reason"] = "frame_data_csv_missing"
        return debug

    min_ts = None
    max_ts = None
    total_rows = 0
    in_range = 0
    missing_files = 0

    try:
        with csv_path.open("r", encoding="utf-8") as handle:
            reader = csv.DictReader(handle)
            for row in reader:
                total_rows += 1
                ts_raw = row.get("Timestamp") or row.get("Time_Seconds") or row.get("Time_Formatted")
                try:
                    if ts_raw is None:
                        continue
                    if isinstance(ts_raw, str) and ":" in ts_raw:
                        parts = [int(p) for p in ts_raw.split(":") if p.strip()]
                        if len(parts) == 3:
                            ts = parts[0] * 3600 + parts[1] * 60 + parts[2]
                        elif len(parts) == 2:
                            ts = parts[0] * 60 + parts[1]
                        else:
                            ts = int(parts[0])
                    else:
                        ts = int(float(ts_raw))
                except Exception:
                    continue

                if min_ts is None or ts < min_ts:
                    min_ts = ts
                if max_ts is None or ts > max_ts:
                    max_ts = ts

                if start <= ts <= end:
                    in_range += 1
                    filename = row.get("Filename") or ""
                    if filename and not (frames_dir / filename).exists():
                        missing_files += 1

        debug.update(
            {
                "total_frames": total_rows,
                "frames_in_range": in_range,
                "min_timestamp": min_ts,
                "max_timestamp": max_ts,
                "missing_frame_files": missing_files,
            }
        )
        if in_range == 0:
            debug["reason"] = "no_frames_in_range"
        elif missing_files > 0:
            debug["reason"] = "missing_frame_files"
        else:
            debug["reason"] = "frames_present"
    except Exception as exc:
        debug["reason"] = "csv_parse_error"
        debug["error"] = str(exc)

    return debug


def _wrap_segment_result(
    video_id: str,
    start: int,
    end: int,
    result_text: Optional[str],
    result_obj: Any,
) -> Any:
    """
    Normalize failed generations into a structured error payload.
    """
    if isinstance(result_obj, dict) and "error" in result_obj:
        return result_obj

    error_info: Optional[Dict[str, Any]] = None

    if result_text is None:
        error_info = {"reason": "generation_returned_none"}
    elif isinstance(result_obj, str):
        error_info = {
            "reason": "invalid_json",
            "raw_preview": result_obj[:300],
        }
    elif isinstance(result_obj, dict) and "questions" not in result_obj:
        error_info = {
            "reason": "missing_questions",
            "keys": list(result_obj.keys()),
        }

    if error_info:
        frame_debug = _segment_frame_debug(video_id, start, end)
        return {"error": error_info, "frame_debug": frame_debug}

    return result_obj


# =========================================================
# Admin page
# =========================================================
@router_admin_pages.get("/", response_class=HTMLResponse)
def admin_page(request: Request):
    # admin.html is self-contained (fetches data via JS), so no heavy context needed
    return templates.TemplateResponse("admin.html", {"request": request})


# =========================================================
# Admin API
# =========================================================
@router_admin_api.post("/download")
async def api_download(url: str = Form(...)):
    # Lazy import to avoid circular dependency
    from main import download_youtube

    outcome = download_youtube(url)
    return outcome


@router_admin_api.post("/frames/{video_id}")
async def api_extract_frames(video_id: str):
    from main import extract_frames_per_second_for_video

    return extract_frames_per_second_for_video(video_id)


@router_admin_api.get("/admin/videos")
def admin_list_downloaded_videos(include_without_frames: bool = False):
    """
    Provide a lightweight manifest of downloaded videos so admins can reuse them.
    """
    videos = _collect_downloaded_videos(include_without_frames=include_without_frames)
    return {
        "success": True,
        "count": len(videos),
        "videos": videos,
        "message": (
            "Videos with extracted frames ready for question generation."
            if not include_without_frames
            else "All downloaded videos."
        ),
    }


@router_admin_api.post("/submit-questions")
async def submit_questions(payload: Dict[str, Any] = Body(...)):
    """
    Submit and save finalized questions (admin 'Submit' in UI).
    Saves to downloads/<video_id>/questions/<video_id>.json
    """
    video_id = payload.get("video_id")
    questions_data = payload.get("questions", [])
    if not video_id or not questions_data:
        raise HTTPException(status_code=400, detail="Missing video_id or questions")

    from datetime import datetime

    questions_dir = DOWNLOADS_DIR / video_id / "questions"
    questions_dir.mkdir(parents=True, exist_ok=True)
    out_path = questions_dir / f"{video_id}.json"

    aggregated = {
        "video_id": video_id,
        "submitted_at": datetime.utcnow().isoformat(),
        "status": "submitted",
        "segments": questions_data,
    }

    try:
        out_path.write_text(
            json.dumps(aggregated, indent=2, ensure_ascii=False), encoding="utf-8"
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to save: {e}")

    return {
        "success": True,
        "message": "Questions submitted successfully",
        "file_url": f"/downloads/{video_id}/questions/{out_path.name}",
        "file_path": str(out_path),
    }


# =========================================================
# WebSocket – keep original path: /ws/questions/{video_id}
# =========================================================
@router_admin_ws.websocket("/ws/questions/{video_id}")
async def ws_questions(websocket: WebSocket, video_id: str):
    await websocket.accept()
    try:
        params = await websocket.receive_json()
        start_seconds = int(params.get("start_seconds", 0))
        interval_seconds = int(params.get("interval_seconds", 60))
        full_duration = bool(params.get("full_duration", False))

        # Lazy imports to avoid circulars
        from main import (
            generate_questions_for_segment_with_retry,
            build_segments_from_duration,
            _maybe_parse_json,
        )
        frames_dir = DOWNLOADS_DIR / video_id / "extracted_frames"
        if not frames_dir.exists():
            await websocket.send_json(
                {
                    "type": "error",
                    "message": "Frames not found. Please extract frames first.",
                }
            )
            await websocket.close()
            return

        # Load duration if available
        duration_seconds = None
        json_path = frames_dir / "frame_data.json"
        if json_path.exists():
            try:
                info = json.loads(json_path.read_text(encoding="utf-8"))
                duration_seconds = int(
                    float(info.get("video_info", {}).get("duration_seconds", 0))
                )
            except Exception:
                duration_seconds = None

        # One-shot interval
        if not full_duration:
            start = max(0, int(start_seconds))
            end = start + max(1, int(interval_seconds)) - 1
            if duration_seconds is not None and end > duration_seconds:
                end = duration_seconds

            await websocket.send_json(
                {
                    "type": "status",
                    "message": f"Generating questions for {start}-{end}s...",
                }
            )
            result_text = await asyncio_to_thread(
                generate_questions_for_segment_with_retry, video_id, start, end
            )
            result_obj = _maybe_parse_json(result_text)
            result_obj = _wrap_segment_result(
                video_id, start, end, result_text, result_obj
            )

            await websocket.send_json(
                {
                    "type": "segment_result",
                    "start": start,
                    "end": end,
                    "result": result_obj,
                }
            )
            await websocket.send_json({"type": "done", "auto_saved": False})
            await websocket.close()
            return

        # Full loop
        if duration_seconds is None or duration_seconds <= 0:
            await websocket.send_json(
                {"type": "error", "message": "Unable to determine video duration."}
            )
            await websocket.close()
            return

        segments = build_segments_from_duration(
            duration_seconds, interval_seconds, start_seconds
        )
        await websocket.send_json(
            {
                "type": "status",
                "message": f"Starting full-duration generation over {len(segments)} segments.",
            }
        )

        aggregated = {
            "video_id": video_id,
            "interval_seconds": int(interval_seconds),
            "start_offset": int(start_seconds),
            "duration_seconds": duration_seconds,
            "segments": [],
        }

        for idx, (seg_start, seg_end) in enumerate(segments, start=1):
            await websocket.send_json(
                {
                    "type": "status",
                    "message": f"[{idx}/{len(segments)}] {seg_start}-{seg_end}s",
                }
            )
            result_text = await asyncio_to_thread(
                generate_questions_for_segment_with_retry,
                video_id,
                seg_start,
                seg_end,
            )
            result_obj = _maybe_parse_json(result_text)
            result_obj = _wrap_segment_result(
                video_id, seg_start, seg_end, result_text, result_obj
            )
            aggregated["segments"].append(
                {"start": seg_start, "end": seg_end, "result": result_obj}
            )
            await websocket.send_json(
                {
                    "type": "segment_result",
                    "start": seg_start,
                    "end": seg_end,
                    "result": result_obj,
                }
            )

        await websocket.send_json(
            {
                "type": "done",
                "segments_count": len(segments),
                "auto_saved": False,
                "data": aggregated,
            }
        )
        await websocket.close()

    except WebSocketDisconnect:
        pass
    except Exception as e:
        try:
            await websocket.send_json({"type": "error", "message": str(e)})
            await websocket.close()
        except Exception:
            pass


# small helper: run sync function in thread (keeps this file standalone)
import asyncio


def asyncio_to_thread(func, *args, **kwargs):
    loop = asyncio.get_event_loop()
    return loop.run_in_executor(None, lambda: func(*args, **kwargs))
