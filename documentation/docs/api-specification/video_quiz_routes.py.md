---
sidebar_position: 4
---

# video_quiz_routes.py

Class: VideoQuizRoutesModule  
Purpose: Handles kids video discovery, final question retrieval, answer grading (RapidFuzz + AI fallback), audio transcription via Whisper, and shared frontend configuration.

Fields:
- router_video_quiz: APIRouter — Router for kids-facing video endpoints (public discovery routes).
- router_api: APIRouter — Router for shared API endpoints (grading, transcription, config).
- DOWNLOADS_DIR: Path — Root directory for downloaded video assets (module invariant: all video data is read from here).
- BASE_DIR: Path — Base project directory (used to write cached JSON).
- GRADING_CONFIG: Dict[str, Any] — Centralized grading thresholds and AI configuration.
- NUM_WORDS: Dict[str, int] — Maps number words to integers (used for numeric extraction).
- SCALE_WORDS: Dict[str, int] — Numeric scale words (hundred, thousand, etc.).
- STOPWORDS: Set[str] — Common words removed during normalization.
- FILLER_WORDS: Set[str] — Speech filler words removed during normalization.
- SYNONYMS: Dict[str, str] — Canonical synonym mappings to normalize meaning.
- OPENAI_CLIENT: OpenAI client — Used for AI grading fallback.
  
Invariants:
- All video metadata is resolved from `DOWNLOADS_DIR/<video_id>`.
- Grading logic follows: Numeric → RapidFuzz → AI fallback → Default.
- AI grading only triggers for borderline similarity scores.

---

## Duration Helpers

- _parse_duration_to_seconds(val: Any): Optional[int] — Converts numeric or time-string duration to seconds.
  Preconditions:
  - `val` must be int, float, or time-formatted string.
  Postconditions:
  - Returns non-negative integer seconds or None.
  Throws:
  - No explicit exceptions (invalid input returns None).
  Example:
  - _parse_duration_to_seconds("01:30") → 90

---

- _format_mmss(sec: Optional[int]): str — Formats seconds into MM:SS.
  Preconditions:
  - `sec` is None or non-negative integer.
  Postconditions:
  - Returns formatted string.
  Example:
  - _format_mmss(125) → "02:05"

---

## Video Discovery

- refresh_kids_videos_json(): List[Dict[str, Any]] — Rebuilds static `kids_videos.json` cache from downloads directory.
  Preconditions:
  - `DOWNLOADS_DIR` exists.
  Postconditions:
  - Scans video folders.
  - Extracts title, duration, thumbnail.
  - Writes `/static/kids_videos.json`.
  Throws:
  - File write errors if disk unavailable.
  Example:
  - refresh_kids_videos_json()

---

- list_kids_videos(): Dict[str, Any] — Returns JSON list of locally available kids videos.
  Preconditions:
  - None.
  Postconditions:
  - Returns `{success, count, videos}`.
  Example:
  - GET `/api/kids_videos`

---

## Final Question Retrieval

- get_final_questions(video_id: str): Dict[str, Any] — Retrieves best-ranked non-trashed question per segment.
  Preconditions:
  - `final_questions.json` exists for `video_id`.
  Postconditions:
  - Selects lowest `llm_ranking`.
  - Skips trashed questions.
  - Excludes final segment from results.
  Throws:
  - Returns error JSON if file missing.
  Example:
  - GET `/api/final-questions/{video_id}`

---

## Answer Grading Helpers

- words_to_numbers(text: str): List[int] — Extracts numeric digits and word-based numbers.
  Preconditions:
  - `text` is string.
  Postconditions:
  - Returns list of detected numbers.
  Example:
  - words_to_numbers("three dogs and 2 cats") → [3, 2]

---

- normalize_text(text: str): str — Lowercases, removes stopwords/fillers, applies synonym mapping.
  Preconditions:
  - `text` is string.
  Postconditions:
  - Returns cleaned canonical string.
  Example:
  - normalize_text("The puppy is happy") → "dog happy"

---

- prepare_text_for_scoring(text: str): str — Cached normalized text with numeric hints appended.
  Preconditions:
  - `text` is string.
  Postconditions:
  - Returns optimized scoring string.
  Throws:
  - None.
  Example:
  - prepare_text_for_scoring("three dogs") → "dog 3"

---

- keyword_overlap(expected: str, user: str): float — Computes keyword overlap ratio.
  Preconditions:
  - Pre-normalized strings.
  Postconditions:
  - Returns similarity ratio [0,1].

---

- simplify_item(item: str): str — Simplifies list item to core keywords.
  Preconditions:
  - `item` is string.
  Postconditions:
  - Returns simplified representation.

---

- extract_items(expected_raw: str): List[str] — Splits comma/and-separated expected answers.
  Preconditions:
  - `expected_raw` contains potential list.
  Postconditions:
  - Returns simplified item list.

---

- list_match(expected_raw: str, user_raw: str): Tuple[int, int, List[str]] — Matches expected list items against user answer.
  Preconditions:
  - Inputs are raw strings.
  Postconditions:
  - Returns (matched_count, total_items, matched_items).

---

- required_items_from_question(question: str, expected: str): int — Determines how many items must be matched.
  Preconditions:
  - Question text may contain numeric words.
  Postconditions:
  - Returns required match count.

---

## Answer Grading Endpoint

- check_answer(payload: Dict[str, Any]): Dict[str, Any] — Grades user answer using numeric checks, RapidFuzz similarity, and optional AI fallback.
  Preconditions:
  - Payload must contain:
    - expected
    - user
    - question
  Postconditions:
  - Returns:
    - similarity score
    - numeric flag
    - status: correct | almost | wrong
    - reason
  Throws:
  - Returns structured JSON for invalid inputs.
  Example:
  - POST `/api/check_answer`

Grading Flow:
1. Numeric validation (exact or mismatch).
2. RapidFuzz similarity scoring.
3. Multi-item partial match logic.
4. AI evaluation for borderline cases.
5. Default fallback.

---

## Audio Transcription

- transcribe_audio(file: UploadFile): Dict[str, Any] — Transcribes audio using Whisper model.
  Preconditions:
  - Valid audio file uploaded.
  Postconditions:
  - Returns `{success, text}` or `{success, error}`.
  Throws:
  - Returns structured error JSON on failure.
  Example:
  - POST `/api/transcribe`

---

## Frontend Configuration

- get_config(): Dict[str, Any] — Returns frontend configuration settings.
  Preconditions:
  - None.
  Postconditions:
  - Returns:
    - skip_prevention flag
    - grading thresholds from `GRADING_CONFIG`
  Example:
  - GET `/api/config`
