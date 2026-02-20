# main.py
from pathlib import Path
from typing import List, Dict, Any, Optional
import base64
import json
import asyncio
from datetime import datetime
from fastapi import (
    FastAPI,
    Form,
    Request,
    Body,
    Query,
    HTTPException,
)
from app.services.video_files import find_primary_video_file
from app.services.expert_review_service import (
    build_expert_preview_data,
    save_expert_annotation_payload,
    get_expert_questions_payload,
    save_expert_question_payload,
    save_final_questions_payload,
)


from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
from app.web import templates

from video_quiz_routes import router_video_quiz, router_api
from admin_routes import router_admin_pages, router_admin_api, router_admin_ws
from app.settings import (
    ADMIN_PASSWORD,
    DOWNLOADS_DIR,
    EXPERT_PASSWORD,
    PUBLIC_ASSETS_DIR,
    EXPERT_QUESTION_TYPE_LABELS,
    EXPERT_QUESTION_TYPES,
    EXPERT_QUESTION_TYPE_VALUES,
)
from app.services.clients import OPENAI_CLIENT


app = FastAPI(title="Piggyback Learning")
app.include_router(router_video_quiz, prefix="/api")  # kids_videos etc
app.include_router(router_api, prefix="/api")  # transcribe, check_answer, config

# Mount admin routers
app.include_router(router_admin_pages, prefix="/admin")
app.include_router(router_admin_api, prefix="/api")
app.include_router(router_admin_ws)

# Serve the downloads directory so the user can click the files
app.mount("/downloads", StaticFiles(directory=str(DOWNLOADS_DIR)), name="downloads")
if PUBLIC_ASSETS_DIR.exists():
    app.mount(
        "/assets",
        StaticFiles(directory=str(PUBLIC_ASSETS_DIR)),
        name="public-assets",
    )

# -----------------------------
@app.get("/", response_class=HTMLResponse)
def home_page(request: Request):
    """Home page with user type selection"""
    return templates.TemplateResponse("home.html", {"request": request})


@app.get("/home", response_class=HTMLResponse)
def home_redirect(request: Request):
    """Alternative home page route"""
    return templates.TemplateResponse("home.html", {"request": request})


@app.get("/children", response_class=HTMLResponse)
def children_page(request: Request):
    """Children's learning interface - no password required"""
    return templates.TemplateResponse("children.html", {"request": request})


@app.post("/api/verify-password")
async def verify_password(
    user_type: str = Form(...), password: str = Form(...)
):
    """Verify password for admin/expert access"""
    valid_passwords = {"admin": ADMIN_PASSWORD, "expert": EXPERT_PASSWORD}

    if user_type in valid_passwords and password == valid_passwords[user_type]:
        if user_type == "admin":
            return JSONResponse({"success": True, "redirect": "/admin"})
        elif user_type == "expert":
            return JSONResponse({"success": True, "redirect": "/expert-preview"})
    else:
        return JSONResponse({"success": False, "message": "Invalid password"})


# -----------------------------
# YouTube Search API (child-safe with duration filters)
# -----------------------------
@app.get("/expert-preview", response_class=HTMLResponse)
def expert_preview(
    request: Request,
    file: Optional[str] = Query(None),
    video: Optional[str] = Query(None),
    mode: Optional[str] = Query("review"),
):
    preview_data = build_expert_preview_data(file=file, video=video, mode=mode)
    context = {
        "request": request,
        **preview_data,
        "question_type_options": [
            {"value": value, "label": label} for value, label in EXPERT_QUESTION_TYPES
        ],
    }
    return templates.TemplateResponse("expert_preview.html", context)


@app.post("/api/expert-annotations")
async def save_expert_annotation(payload: Dict[str, Any] = Body(...)):
    result = save_expert_annotation_payload(payload)
    return JSONResponse(result)


@app.get("/api/videos-list")
async def list_videos():
    """List all downloaded videos with title, thumbnail, duration, and question counts."""
    try:
        videos = []
        if not DOWNLOADS_DIR.exists():
            return JSONResponse({"success": True, "videos": []})

        for video_dir in sorted(DOWNLOADS_DIR.iterdir()):
            if not video_dir.is_dir():
                continue

            video_id = video_dir.name
            meta_path = video_dir / "meta.json"
            meta_data = {}

            if meta_path.exists():
                try:
                    meta_data = json.loads(meta_path.read_text(encoding="utf-8"))
                except Exception:
                    meta_data = {}

            title = meta_data.get("title", video_id)
            thumbnail = meta_data.get("thumbnail", "")
            duration = meta_data.get("duration", 0)

            video_file = find_primary_video_file(video_dir)
            if not video_file:
                continue

            questions_dir = video_dir / "questions"
            question_files = []
            if questions_dir.exists():
                question_files = [
                    p for p in questions_dir.glob("*.json") if p.is_file()
                ]

            question_count = len(question_files)

            # Create video URL
            video_url = f"/downloads/{video_file.relative_to(DOWNLOADS_DIR).as_posix()}"

            videos.append(
                {
                    "id": video_id,
                    "title": title,
                    "thumbnail": thumbnail,
                    "duration": duration,
                    "videoUrl": video_url,
                    "questionCount": question_count,
                }
            )

        return JSONResponse({"success": True, "videos": videos})

    except Exception as e:
        return JSONResponse(
            {"success": False, "message": f"Error listing videos: {e}", "videos": []}
        )


@app.get("/api/expert-questions/{video_id}")
async def get_expert_questions(video_id: str):
    result, status_code = get_expert_questions_payload(video_id)
    return JSONResponse(result, status_code=status_code)

@app.post("/api/expert-questions")
async def save_expert_question(payload: Dict[str, Any] = Body(...)):
    result, status_code = save_expert_question_payload(payload)
    return JSONResponse(result, status_code=status_code)

@app.post("/api/save-final-questions")
async def save_final_questions(payload: Dict[str, Any] = Body(...)):
    result, status_code = save_final_questions_payload(payload)
    return JSONResponse(result, status_code=status_code)


@app.post("/api/tts")
async def synthesize_tts(payload: Dict[str, Any] = Body(...)):
    """Generate speech audio via OpenAI TTS."""
    text = str(payload.get("text") or "").strip()
    if not text:
        return JSONResponse(
            {"success": False, "message": "text is required"}, status_code=400
        )

    voice = str(payload.get("voice") or "sage").strip() or "sage"
    raw_speed = payload.get("speed", 0.75)
    try:
        speed = float(raw_speed)
    except (TypeError, ValueError):
        speed = 0.75
    speed = max(0.25, min(speed, 4.0))
    response_format = str(payload.get("format") or "mp3").strip() or "mp3"

    def _synthesize(voice_name: str) -> bytes:
        with OPENAI_CLIENT.audio.speech.with_streaming_response.create(
            model="gpt-4o-mini-tts",
            voice=voice_name,
            input=text,
            speed=speed,
        ) as response:
            return response.read()

    try:
        audio_bytes = await asyncio.to_thread(_synthesize, voice)
    except Exception as exc:
        # Attempt a graceful fallback if the requested voice is unavailable.
        fallback_voice = "alloy"
        error_message = str(exc)
        should_retry_with_fallback = (
            voice.lower() != fallback_voice
            and any(
                keyword in error_message.lower()
                for keyword in ("voice", "unknown", "not found", "unsupported")
            )
        )
        if should_retry_with_fallback:
            try:
                audio_bytes = await asyncio.to_thread(_synthesize, fallback_voice)
                voice = fallback_voice
            except Exception as retry_exc:
                error_message = f"{error_message} | fallback_failed={retry_exc}"
                return JSONResponse(
                    {
                        "success": False,
                        "message": f"TTS generation failed: {error_message}",
                    },
                    status_code=502,
                )
        else:
            return JSONResponse(
                {
                    "success": False,
                    "message": f"TTS generation failed: {error_message}",
                },
                status_code=502,
            )
    audio_b64 = base64.b64encode(audio_bytes).decode("utf-8")
    return JSONResponse(
        {"success": True, "audio": audio_b64, "format": response_format, "voice": voice}
    )


app.mount("/static", StaticFiles(directory="static"), name="static")
