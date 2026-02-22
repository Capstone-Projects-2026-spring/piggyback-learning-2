---
sidebar_position: 2
---

# main.py

Class: FastAPI Application (`app`)
Purpose: Entry point and HTTP interface for the Piggyback Learning platform. Configures routes, mounts static directories, integrates admin/expert APIs, and exposes TTS and video management endpoints.

Fields:
  - app: FastAPI — Main ASGI application instance (global singleton, required for server startup)

Methods:

  - home_page(request: Request): HTMLResponse — Renders the main home page.
    Preconditions:
      - Templates directory must be configured correctly.
    Postconditions:
      - Returns rendered `home.html`.
    Throws:
      - Template-related exceptions if template is missing.
    Example:
      - GET `/`

  - home_redirect(request: Request): HTMLResponse — Alternative route for home page.
    Preconditions:
      - Templates configured correctly.
    Postconditions:
      - Returns rendered `home.html`.
    Throws:
      - Template-related exceptions if template is missing.
    Example:
      - GET `/home`

  - children_page(request: Request): HTMLResponse — Renders children learning interface (no password required).
    Preconditions:
      - Templates configured correctly.
    Postconditions:
      - Returns rendered `children.html`.
    Example:
      - GET `/children`

  - verify_password(user_type: str, password: str): JSONResponse — Validates admin or expert password.
    Preconditions:
      - `user_type` must be `"admin"` or `"expert"`.
      - `password` must be provided via form submission.
    Postconditions:
      - Returns JSON `{ success: True, redirect: <route> }` if valid.
      - Returns JSON `{ success: False }` if invalid.
    Throws:
      - No explicit exceptions (handled via JSON failure response).
    Example:
      - POST `/api/verify-password`

  - expert_preview(request: Request, file: Optional[str], video: Optional[str], mode: Optional[str]): HTMLResponse — Renders expert preview interface.
    Preconditions:
      - Preview data must be buildable via `build_expert_preview_data`.
    Postconditions:
      - Returns rendered `expert_preview.html` with:
        - Video preview data
        - Question type options
    Throws:
      - Exceptions from preview data builder if input invalid.
    Example:
      - GET `/expert-preview?video=abc123&mode=review`

  - save_expert_annotation(payload: Dict[str, Any]): JSONResponse — Persists expert annotation payload.
    Preconditions:
      - JSON body must match expected annotation schema.
    Postconditions:
      - Returns result from `save_expert_annotation_payload`.
    Throws:
      - Service-layer validation exceptions wrapped in response.
    Example:
      - POST `/api/expert-annotations`

  - list_videos(): JSONResponse — Lists downloaded videos with metadata and question counts.
    Preconditions:
      - `DOWNLOADS_DIR` must exist or be accessible.
    Postconditions:
      - Returns JSON:
        ```
        {
          success: bool,
          videos: [
            {
              id,
              title,
              thumbnail,
              duration,
              videoUrl,
              questionCount
            }
          ]
        }
        ```
    Throws:
      - Returns failure JSON if filesystem or parsing error occurs.
    Example:
      - GET `/api/videos-list`

  - get_expert_questions(video_id: str): JSONResponse — Retrieves saved expert questions for a video.
    Preconditions:
      - `video_id` must exist.
    Postconditions:
      - Returns JSON payload with appropriate status code.
    Throws:
      - Propagates status from `get_expert_questions_payload`.
    Example:
      - GET `/api/expert-questions/{video_id}`

  - save_expert_question(payload: Dict[str, Any]): JSONResponse — Saves a single expert question.
    Preconditions:
      - JSON body must match expected question schema.
    Postconditions:
      - Returns service-layer result with status code.
    Throws:
      - Validation or persistence errors via service response.
    Example:
      - POST `/api/expert-questions`

  - save_final_questions(payload: Dict[str, Any]): JSONResponse — Persists finalized set of questions for a video.
    Preconditions:
      - JSON body must match expected schema.
    Postconditions:
      - Returns service-layer result with status code.
    Throws:
      - Validation or persistence errors via service response.
    Example:
      - POST `/api/save-final-questions`

  - synthesize_tts(payload: Dict[str, Any]): JSONResponse — Generates text-to-speech audio using OpenAI TTS.
    Preconditions:
      - `text` field must be present and non-empty.
      - Optional fields: `voice`, `speed`, `format`.
    Postconditions:
      - Returns JSON:
        ```
        {
          success: True,
          audio: <base64-encoded audio>,
          format: "mp3",
          voice: <voice_used>
        }
        ```
      - On failure returns:
        ```
        {
          success: False,
          message: <error message>
        }
        ```
    Throws:
      - Returns HTTP 400 if `text` missing.
      - Returns HTTP 502 if TTS generation fails.
    Behavior:
      - Clamps speed to range [0.25, 4.0].
      - Attempts fallback voice `"alloy"` if requested voice fails.
      - Runs blocking OpenAI call in thread using `asyncio.to_thread`.
    Example:
      - POST `/api/tts`
        ```
        {
          "text": "Hello world",
          "voice": "sage",
          "speed": 1.0,
          "format": "mp3"
        }
        ```

Additional Configuration:

  - Router Registration:
      - Includes video quiz routes under `/api`
      - Includes admin pages under `/admin`
      - Includes admin APIs under `/api`
      - Includes admin WebSocket routes

  - Static Mounts:
      - `/downloads` → serves video files from `DOWNLOADS_DIR`
      - `/assets` → serves public assets if directory exists
      - `/static` → serves static frontend resources

Invariants:
  - `app` must be instantiated exactly once.
  - Mounted directories must remain accessible during runtime.
  - Service-layer functions handle business logic; this module only orchestrates HTTP layer.
