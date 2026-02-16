---
sidebar_position: 1
---

# design

## Purpose
This document describes the **software architecture** for Piggyback Learning and shows how our requirements map to the design. It focuses on the **components, their responsibilities, interfaces, and data flow** (high-level).  
> Note: our **system block diagram** and **sequence diagrams** are documented on their dedicated pages, and are referenced here rather than duplicated.

## Architecture Overview (High-Level)
Piggyback Learning is a **web application** with a FastAPI backend that serves pages and APIs for three user experiences:

- **Child**: search/select a video, watch it, answer questions, receive feedback.
- **Expert**: review AI-generated questions, approve/edit/skip, and record “best question” feedback.
- **Admin**: manage configuration and monitor outputs (basic access-controlled routes).

We keep the architecture intentionally simple (≤ 7 components) and integrate external tools/services where appropriate (YouTube extraction, FFmpeg, OpenAI/Gemini).

### Related diagrams
- System Block Diagram: see `System Architecture → system-block-diagram`
- Sequence Diagrams (use cases): see `System Architecture → Sequence Diagrams`
- Class diagrams: see `System Architecture → Class Diagrams`

---

## Design Entities (Checklist-Friendly)

Below are the primary design entities with identification, purpose, function, dependencies, interfaces, processing, and data.

### 1) Web Client (Browser UI)
- **Type:** Client / Frontend (server-rendered templates + static assets)
- **Purpose:** Provide usable experiences for Child / Expert / Admin.
- **Function:** Renders pages, sends REST requests, optionally uses WebSockets for streaming updates.
- **Dependencies:** Backend API, browser runtime.
- **Interface:** HTTP (pages) + REST calls to `/api/*` + optional WebSocket endpoint(s).
- **Processing:** User input, form submissions, polling/updates, displaying video & Q/A.
- **Data (hidden inside):** Session state in browser; no sensitive secrets stored client-side.

### 2) API Server (FastAPI “Piggyback Learning” App)
- **Type:** Backend service
- **Purpose:** Central orchestrator for downloading videos, extracting frames/subtitles, generating questions, and saving review outputs.
- **Function:** Routes requests, validates payloads, coordinates external services, reads/writes project data.
- **Dependencies:** `video_quiz_routes`, `admin_routes`, filesystem storage, yt-dlp, FFmpeg, OpenAI/Gemini.
- **Interface:**
  - **Pages:**  `/children`, `/admin/*`, `/expert-preview`
  - **REST APIs:** `/api/*` (examples below)
  - **WebSocket:** (used for streaming interval results if enabled)
- **Processing:** Request validation, orchestration, retry logic, JSON normalization, file persistence.
- **Data (hidden inside):** Environment-configured secrets (API keys, passwords), runtime caches (clients/config).

**Key API endpoints (examples):**
- `POST /api/verify-password` – gates admin/expert access
- `GET /api/videos-list` – lists downloaded videos + metadata
- `POST /api/save-final-questions` – persists final reviewed questions
- `POST /api/tts` – generates speech audio for text (OpenAI TTS)
- `POST /api/expert-annotations` – stores expert review/approval notes

### 3) Video Acquisition Module (yt-dlp + helpers)
- **Type:** Integration component
- **Purpose:** Download a YouTube video + metadata and make it available locally.
- **Function:** Extract metadata (title, thumbnail, duration), download video, optionally download subtitles.
- **Dependencies:** `yt_dlp`, optional `node` runtime, optional cookies file, optional `ffmpeg` for remuxing.
- **Interface:** Internal Python functions (called by API server).
- **Processing:** Attempts multiple “player clients” on 403 errors; format fallback when requested formats are unavailable.
- **Data (hidden inside):** Download options, cookies path, downloaded files.

### 4) Media Processing Module (Frames/Subtitles)
- **Type:** Processing component
- **Purpose:** Convert video into structured inputs (frames and transcript snippets) for question generation.
- **Function:** Extract 1 frame per second, store frame index + timestamps; collect transcript lines for segment windows.
- **Dependencies:** OpenCV (`cv2`), PIL, pandas; relies on downloaded video file and subtitle availability.
- **Interface:** Internal functions (called by API server).
- **Processing:** Video read → frame sampling → write CSV/JSON summaries.
- **Data (hidden inside):** Frame cache, frame metadata, transcript window text.

### 5) Question Generation Engine (LLM Provider: OpenAI/Gemini)
- **Type:** External service integration
- **Purpose:** Generate child-friendly comprehension questions from transcript + sampled frames.
- **Function:** For each segment, returns JSON containing 7 categories (character/setting/feeling/action/causal/outcome/prediction) + ranking + best question.
- **Dependencies:** OpenAI API client, Gemini API client, environment keys, retry/backoff logic.
- **Interface:** Internal function that returns **JSON text** (schema enforced).
- **Processing:** Prompt creation → send frames + transcript → validate JSON → retry on rate limit/invalid JSON.
- **Data (hidden inside):** Provider selection, model name, raw provider response logs (if any).

### 6) Review & Annotation Module (Expert Workflow)
- **Type:** Application module
- **Purpose:** Allow experts to approve/edit/skip AI questions and record rankings/feedback.
- **Function:** Loads generated questions, shows per-segment questions, saves expert annotations and “best question” approval.
- **Dependencies:** Filesystem storage, API routes, templates.
- **Interface:** UI: `/expert-preview` + API: `/api/expert-annotations`, `/api/expert-questions/*`
- **Processing:** Merge/overwrite annotations by (start,end) segment, preserve best-question feedback, keep sorted.
- **Data (hidden inside):** Expert annotations JSON bundles.

### 7) Storage Layer (Filesystem-based Data Store)
- **Type:** Data component
- **Purpose:** Persist all artifacts without requiring a DB (simple + debuggable).
- **Function:** Stores downloaded videos, metadata, extracted frames, generated question JSON, expert annotations, and final reviewed outputs.
- **Dependencies:** OS filesystem.
- **Interface:** Directory layout + JSON/CSV schemas.
- **Processing:** Write/read JSON files, build downloads URLs, ensure folders exist.
- **Data (hidden inside):** Files under `downloads/<video_id>/...`

**Primary stored artifacts (per video):**
- `downloads/<video_id>/meta.json` – title/thumbnail/duration + local path
- `downloads/<video_id>/<video_file>.mp4|webm|mkv` – video asset
- `downloads/<video_id>/extracted_frames/frame_*.jpg` – sampled frames
- `downloads/<video_id>/extracted_frames/frame_data.csv` + `frame_data.json` – frame metadata
- `downloads/<video_id>/questions/*.json` – AI-generated segment questions
- `downloads/<video_id>/questions/<file>.expert.json` – expert annotations (review mode)
- `downloads/<video_id>/expert_questions/expert_questions.json` – expert-created questions (create mode)
- `downloads/<video_id>/final_questions/final_questions.json` – finalized reviewed output

---

## Requirements → Architecture Mapping (Traceability)

- **Generate questions for children**  
  → Question Generation Engine + Media Processing Module + API Server orchestration.

- **Support multiple user roles (Child / Expert / Admin)**  
  → Web Client pages + API Server routing; password verification for expert/admin routes.

- **Provide feedback + track review decisions**  
  → Review & Annotation Module storing structured expert annotations and “best question” approval.

- **Reliability under rate limits / failures**  
  → Retry/backoff in question generation; format/client fallbacks for downloads; transcript-only fallback.

- **Maintainability / simplicity**  
  → Fewer than 7 high-level components; external integrations isolated behind helper functions.

---

## Interfaces and Protocols (Enough to Implement Independently)

### HTTP Routes (High-level)
- **Pages:** `GET /`, `GET /children`, `GET /expert-preview`, `GET /admin/*`
- **REST:** `GET/POST /api/*`
- **WebSocket:** Used when streaming progress/results is enabled.

### Auth / Access Control
- Basic role gating via `POST /api/verify-password` using environment-configured passwords for Admin/Expert.  
- No user PII stored; keys/passwords are loaded from environment variables (`.env` / `.env.txt`) and never committed.

---

## Achievability & Integration Plan
The system is achievable with off-the-shelf components:
- FastAPI for backend + templates/static assets for UI
- yt-dlp + FFmpeg for robust video handling
- OpenAI/Gemini for question generation and TTS
- Filesystem storage for all artifacts (no DB required)

If we need scalability later, the storage layer can be upgraded to an object store + database without changing the UI contracts (keep the `/api/*` schemas stable).

---

## Database
Currently, we use a **filesystem-based store** instead of a relational database.  
If a database is introduced later, we will add:
- ER diagram + table design for Videos, Segments, Questions, ExpertAnnotations, FinalQuestions
- Migration plan from `downloads/<video_id>/*` JSON to relational tables
