---
sidebar_position: 3
---

# admin_routes.py

Class: AdminRoutesModule  
Purpose: Defines all admin-related HTTP routes, API endpoints, and WebSocket handlers for managing downloaded videos, extracted frames, and AI-generated questions.

Fields:
- templates: Jinja2Templates — Template renderer configured with shared `TEMPLATES_DIR` (module-level, invariant: must match app-wide template path)
- router_admin_pages: APIRouter — Router for admin HTML pages (mounted under `/admin`)
- router_admin_api: APIRouter — Router for admin REST API endpoints (mounted under `/api`)
- router_admin_ws: APIRouter — Router for admin WebSocket endpoints (mounted with no prefix)
- DOWNLOADS_DIR: Path — Shared directory containing downloaded video assets (imported from settings; invariant: must remain consistent across modules)

Methods:

---

- format_hhmmss(total_seconds: int): str — Formats seconds into `HH:MM:SS` or `MM:SS`.
  Preconditions:
  - `total_seconds` must be a non-negative integer.
  Postconditions:
  - Returns formatted time string.
  Throws:
  - No explicit exceptions (invalid input may raise TypeError).
  Example:
  - format_hhmmss(125) → `"02:05"`

---

- _collect_downloaded_videos(include_without_frames: bool): List[Dict[str, Any]] — Enumerates downloaded videos and collects metadata for admin UI.
  Preconditions:
  - `DOWNLOADS_DIR` must exist and contain `<video_id>` subdirectories.
  Postconditions:
  - Returns a list of video metadata dictionaries including:
    - video_id
    - title
    - duration
    - frame availability
    - question file availability
  Throws:
  - Silently ignores malformed metadata files.
  Example:
  - _collect_downloaded_videos(False) → `[ { "video_id": "...", "has_frames": True } ]`

---

- _segment_frame_debug(video_id: str, start: int, end: int): Dict[str, Any] — Inspects frame coverage for a specific segment.
  Preconditions:
  - `video_id` must correspond to a folder in `DOWNLOADS_DIR`.
  - `start` and `end` define a valid time interval.
  Postconditions:
  - Returns debug dictionary including:
    - frame counts
    - timestamp range
    - missing frame files
    - failure reason if applicable
  Throws:
  - Returns structured error payload instead of raising.
  Example:
  - _segment_frame_debug("abc123", 0, 60)

---

- _wrap_segment_result(video_id: str, start: int, end: int, result_text: Optional[str], result_obj: Any): Any — Normalizes question generation results.
  Preconditions:
  - `result_text` is raw generation output.
  - `result_obj` is parsed JSON or original object.
  Postconditions:
  - Returns valid question object OR structured error payload including frame debug info.
  Throws:
  - No direct exceptions (wraps failures).
  Example:
  - _wrap_segment_result("abc123", 0, 60, raw, parsed)

---

- admin_page(request: Request): HTMLResponse — Renders the admin dashboard page.
  Preconditions:
  - `admin.html` template must exist.
  Postconditions:
  - Returns rendered template.
  Throws:
  - Template rendering exceptions if missing.
  Example:
  - GET `/admin/`

---

- api_download(url: str): Dict[str, Any] — Downloads a YouTube video.
  Preconditions:
  - `url` must be a valid YouTube URL.
  Postconditions:
  - Returns download outcome from service layer.
  Throws:
  - Service-layer exceptions may propagate.
  Example:
  - POST `/api/download`

---

- api_extract_frames(video_id: str): Dict[str, Any] — Extracts frames per second for a video.
  Preconditions:
  - `video_id` must exist in `DOWNLOADS_DIR`.
  Postconditions:
  - Returns extraction result from frame service.
  Throws:
  - Service-layer exceptions may propagate.
  Example:
  - POST `/api/frames/{video_id}`

---

- admin_list_downloaded_videos(include_without_frames: bool): Dict[str, Any] — Returns lightweight manifest of downloaded videos.
  Preconditions:
  - `DOWNLOADS_DIR` accessible.
  Postconditions:
  - Returns JSON:
    - success flag
    - count
    - list of videos
  Throws:
  - No explicit exceptions.
  Example:
  - GET `/api/admin/videos`

---

- submit_questions(payload: Dict[str, Any]): Dict[str, Any] — Saves finalized questions for a video.
  Preconditions:
  - `payload` must contain `video_id` and `questions`.
  Postconditions:
  - Writes JSON to:
    `downloads/<video_id>/questions/<video_id>.json`
  Throws:
  - HTTPException(400) if missing required fields.
  - HTTPException(500) if file write fails.
  Example:
  - POST `/api/submit-questions`

---

- ws_questions(websocket: WebSocket, video_id: str): None — WebSocket endpoint for streaming AI question generation.
  Preconditions:
  - Frames must exist for the video.
  - Client must send JSON including:
    - start_seconds
    - interval_seconds
    - full_duration
  Postconditions:
  - Streams:
    - status updates
    - per-segment results
    - final aggregated result
  Throws:
  - Sends structured error messages on failure.
  - Handles WebSocketDisconnect silently.
  Example:
  - WS `/ws/questions/{video_id}`

---

- asyncio_to_thread(func, *args, **kwargs): Future — Runs synchronous function in thread executor.
  Preconditions:
  - `func` must be callable.
  Postconditions:
  - Returns Future resolving to function result.
  Throws:
  - Exceptions from `func` propagate to caller.
  Example:
  - await asyncio_to_thread(my_function, arg1)

---

Invariants:
- All file operations are relative to `DOWNLOADS_DIR`.
- Routers are mounted externally in main application.
- Business logic is delegated to service-layer modules.
- WebSocket question generation depends on extracted frame data.
