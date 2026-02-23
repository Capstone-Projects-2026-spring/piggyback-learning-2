---
title: Internal Code Contract
sidebar_position: 3
description: Core backend service contracts (implementation-level).
---

# Internal Code Contract (Python)

This page documents internal service contracts that implement the backend API.

## `app/services/frame_service.py`

### `extract_frames_per_second_for_video(video_id: str) -> Dict[str, Any]`
- Purpose: Extract 1 frame per second from `downloads/<video_id>/` and persist frame metadata.
- Parameters: `video_id` (video folder id).
- Returns: dict with `success`, `message`, `files`, `video_id`, `output_dir`, `count`.
- Preconditions: video folder exists and contains a supported video file.
- Postconditions: writes `extracted_frames/`, `frame_data.json`, `frame_data.csv`.
- Error behavior: returns structured failure payload for missing folder/video/FPS/read issues.

## `app/services/question_generation_service.py`

### `encode_image_to_base64(image_path, max_size=(512, 512)) -> Optional[str]`
- Purpose: Convert frame image to resized base64 JPEG.
- Returns: base64 string or `None` on error.

### `time_to_seconds(time_str) -> int`
- Purpose: Convert `HH:MM:SS` / `MM:SS` / seconds text to integer seconds.
- Returns: parsed seconds or `0` on invalid input.

### `read_frame_data_from_csv(folder_name, start_time, end_time) -> Tuple[List[Dict[str, Any]], str]`
- Purpose: Load and filter frame rows for a segment; build transcript text.
- Returns: `(frame_data, complete_transcript)`.

### `generate_questions_for_segment(video_id, start_time, end_time, polite_first=False, provider=None) -> Optional[str]`
- Purpose: Generate segment question JSON from frames + transcript via LLM provider.
- Returns: JSON text on success, JSON error payload text for known failures, or `None`.
- Preconditions: frames/transcript exist; provider credentials configured.

### `generate_questions_for_segment_with_retry(video_id, start_time, end_time, max_attempts=10, provider=None) -> Optional[str]`
- Purpose: Retry orchestration for segment generation.
- Returns: successful JSON text or final failure result.

### `build_segments_from_duration(duration_seconds, interval_seconds, start_offset=0) -> List[tuple]`
- Purpose: Build inclusive segment windows `(start, end)` over duration.

### `_maybe_parse_json(text)`
- Purpose: Best-effort parser for model output (including fenced JSON).
- Returns: parsed object or raw text.

### `persist_segment_questions_json(video_id, start, end, payload) -> Optional[str]`
- Purpose: Save one segment’s question payload into `downloads/<video_id>/questions/`.
- Returns: downloads URL or `None` on failure.

### `resolve_question_file_param(value) -> Optional[Path]`
- Purpose: Safely resolve user-supplied question JSON path under `DOWNLOADS_DIR`.
- Returns: resolved JSON `Path` or `None` if invalid/unsafe.

## `app/services/download_service.py`

### `download_youtube(url: str) -> Dict[str, Any]`
- Purpose: Download YouTube video, gather metadata, and persist `meta.json`.
- Returns: normalized result dict (`success`, `message`, `video_id`, `title`, `thumbnail`, `files`, optional `duration`, `local_path`).
- Preconditions: valid YouTube URL.
- Postconditions: creates/updates `downloads/<video_id>/` assets.
- Error behavior: returns structured failure payload (no unhandled route-level exception expected).

## `app/services/expert_review_service.py`

### `build_expert_preview_data(file, video, mode) -> Dict[str, Any]`
- Purpose: Build expert preview context (segments, selected file/video, annotation state).

### `save_expert_annotation_payload(payload) -> Dict[str, Any]`
- Purpose: Save/update expert annotation for segment.
- Error behavior: raises `HTTPException(400/500)` on validation/persistence failure.

### `get_expert_questions_payload(video_id) -> Tuple[Dict[str, Any], int]`
- Purpose: Load stored expert questions for video.

### `save_expert_question_payload(payload) -> Tuple[Dict[str, Any], int]`
- Purpose: Upsert expert question (or skipped segment marker).

### `save_final_questions_payload(payload) -> Tuple[Dict[str, Any], int]`
- Purpose: Persist final ranked questions to `final_questions.json`.

## `app/services/clients.py`

### `get_openai_client() -> OpenAI`
- Purpose: Build cached OpenAI client from environment.
- Preconditions: `OPENAI_API_KEY` exists.
- Error behavior: raises `RuntimeError` if key missing.

## Route-to-Service Traceability

- `POST /api/download` -> `download_youtube`
- `POST /api/frames/{video_id}` -> `extract_frames_per_second_for_video`
- `WS /ws/questions/{video_id}` -> segment generation functions in `question_generation_service.py`
- `POST /api/expert-annotations` -> `save_expert_annotation_payload`
- `GET /api/expert-questions/{video_id}` -> `get_expert_questions_payload`
- `POST /api/expert-questions` -> `save_expert_question_payload`
- `POST /api/save-final-questions` -> `save_final_questions_payload`
