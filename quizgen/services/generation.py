import json
import os
import random
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from django.conf import settings
from google import genai
from google.genai import types

from videos.models import ExtractedFrame, Video


def get_gemini_client():
    api_key = os.getenv('GEMINI_API_KEY', '').strip()
    if not api_key:
        raise RuntimeError('GEMINI_API_KEY is not set')

    return genai.Client(
        api_key=api_key,
        http_options=types.HttpOptions(
            api_version=os.getenv('GEMINI_API_VERSION', 'v1')
        ),
    )


def _sample_frames(
    frames: List[ExtractedFrame], max_frames: int = 5
) -> List[ExtractedFrame]:
    if len(frames) <= max_frames:
        return frames
    step = max(1, len(frames) // max_frames)
    return frames[0::step][:max_frames]


def _build_transcript_from_frames(frames: List[ExtractedFrame]) -> str:
    parts = []
    for fr in frames:
        txt = (fr.subtitle_text or '').strip()
        if txt:
            label = fr.timestamp_formatted or str(fr.timestamp_seconds)
            parts.append(f'[{label}] {txt}')
    return (
        '\n'.join(parts) if parts else 'No transcript available for this video segment.'
    )


def generate_questions_for_segment(
    video_id: str,
    start_time: int,
    end_time: int,
    polite_first: bool = False,
) -> str:
    """
    DB-backed generation using ExtractedFrame rows, Gemini-only.
    Returns JSON text (or JSON error string).
    """
    video = Video.objects.get(id=video_id)

    frames = list(
        ExtractedFrame.objects.filter(
            video=video,
            timestamp_seconds__gte=start_time,
            timestamp_seconds__lte=end_time,
        ).order_by('timestamp_seconds')
    )
    if not frames:
        return json.dumps(
            {'error': {'reason': 'no_frames_in_segment', 'retryable': False}}
        )

    if not getattr(settings, 'GEMINI_API_KEY', '') and not os.getenv(
        'GOOGLE_API_KEY', ''
    ):
        return json.dumps(
            {'error': {'reason': 'gemini_key_missing', 'retryable': False}}
        )

    transcript = _build_transcript_from_frames(frames)
    duration = end_time - start_time + 1

    system_message = (
        'You are a safe, child-focused educational assistant. '
        "The content is a children's educational video. "
        'Follow all safety policies and avoid disallowed content. '
        'Provide age-appropriate, neutral, factual responses only.'
    )

    base_prompt = f"""You are an early childhood educator designing comprehension questions for children ages 6–8.
Analyze the video content using both the visual frames and the complete transcript provided below.

COMPLETE TRANSCRIPT:
==========================================
{transcript}
==========================================

TASK:
I am providing you with sequential frames from a {duration}-second segment ({start_time}s to {end_time}s).

1. Provide ONE short, child-friendly comprehension question for EACH category:
   - Character
   - Setting
   - Feeling
   - Action
   - Causal Relationship
   - Outcome
   - Prediction

2. Rank the questions (best = 1)

3. Return JSON only in this structure:
{{
  "questions": {{
    "character": {{ "q": "...", "a": "...", "rank": "" }},
    "setting": {{ "q": "...", "a": "...", "rank": "" }},
    "feeling": {{ "q": "...", "a": "...", "rank": "" }},
    "action": {{ "q": "...", "a": "...", "rank": "" }},
    "causal": {{ "q": "...", "a": "...", "rank": "" }},
    "outcome": {{ "q": "...", "a": "...", "rank": "" }},
    "prediction": {{ "q": "...", "a": "...", "rank": "" }}
  }},
  "best_question": "..."
}}
"""

    polite_prompt = f"""You are helping create educational questions for young children. This is a children's educational video with no violence or inappropriate content.

COMPLETE TRANSCRIPT:
==========================================
{transcript}
==========================================

I am providing you with sequential frames from a {duration}-second segment ({start_time}s to {end_time}s).

Create ONE short, child-friendly comprehension question for EACH category:
- Character
- Setting
- Feeling
- Action
- Causal Relationship
- Outcome
- Prediction

Rank the questions (best = 1)

Return JSON only in this structure:
{{
  "questions": {{
    "character": {{ "q": "...", "a": "...", "rank": "" }},
    "setting": {{ "q": "...", "a": "...", "rank": "" }},
    "feeling": {{ "q": "...", "a": "...", "rank": "" }},
    "action": {{ "q": "...", "a": "...", "rank": "" }},
    "causal": {{ "q": "...", "a": "...", "rank": "" }},
    "outcome": {{ "q": "...", "a": "...", "rank": "" }},
    "prediction": {{ "q": "...", "a": "...", "rank": "" }}
  }},
  "best_question": "..."
}}
"""

    sampled = _sample_frames(frames, max_frames=5)

    # Build absolute image paths from ExtractedFrame.file_path
    image_paths: List[Path] = []
    for fr in sampled:
        try:
            rel = fr.file_path.replace('downloads/', '', 1)
            image_paths.append(Path(settings.DOWNLOADS_DIR) / rel)
        except Exception:
            continue

    def _call_gemini(prompt_text: str) -> Optional[str]:
        """
        Uses google-genai SDK.
        Sends system instruction + prompt + images, requests JSON response.
        """
        try:
            client = get_gemini_client()
        except Exception as exc:
            return json.dumps(
                {
                    'error': {
                        'reason': 'gemini_client_error',
                        'retryable': False,
                        'message': str(exc),
                    }
                }
            )

        model_name = getattr(
            settings,
            'GEMINI_MODEL',
            os.getenv('GEMINI_MODEL', 'gemini-2.0-flash'),
        )

        parts: List[types.Part] = [types.Part.from_text(text=prompt_text)]

        added_image = False
        for p in image_paths:
            try:
                # Infer mime type from extension (default to jpeg)
                suffix = p.suffix.lower()
                mime = 'image/jpeg'
                if suffix == '.png':
                    mime = 'image/png'
                elif suffix in ('.webp',):
                    mime = 'image/webp'

                image_bytes = p.read_bytes()
                parts.append(types.Part.from_bytes(data=image_bytes, mime_type=mime))
                added_image = True
            except Exception:
                continue

        if not added_image:
            return json.dumps(
                {'error': {'reason': 'frame_encoding_failed', 'retryable': False}}
            )

        resp = client.models.generate_content(
            model=model_name,
            contents=parts,
            config=types.GenerateContentConfig(
                system_instruction=system_message,
                temperature=0.3,
                max_output_tokens=1500,
                response_mime_type='application/json',
            ),
        )
        return getattr(resp, 'text', None)

    prompt_sequence = [base_prompt, polite_prompt]
    if polite_first:
        prompt_sequence = [polite_prompt, base_prompt]

    last_error: Optional[Dict[str, Any]] = None
    tried_transcript_only = False

    # Similar retry behaviour to your previous version
    for prompt in prompt_sequence:
        for attempt in range(3):
            out = _call_gemini(prompt)
            if out:
                try:
                    json.loads(out)
                    return out
                except Exception:
                    last_error = {
                        'reason': 'invalid_json',
                        'retryable': True,
                        'raw_preview': str(out)[:300],
                    }
            else:
                last_error = {'reason': 'empty_response', 'retryable': True}

            time.sleep(random.uniform(0.5, 1.5))

        # transcript-only fallback once if we got empty responses
        if (
            not tried_transcript_only
            and last_error
            and last_error.get('reason') == 'empty_response'
        ):
            tried_transcript_only = True
            out = _call_gemini(
                prompt
                + '\n\nIf visuals are unavailable, answer using the transcript only.'
            )
            if out:
                try:
                    json.loads(out)
                    return out
                except Exception:
                    last_error = {
                        'reason': 'invalid_json',
                        'retryable': True,
                        'raw_preview': str(out)[:300],
                    }

    return json.dumps(
        {'error': last_error or {'reason': 'generation_failed', 'retryable': True}}
    )


def generate_questions_for_segment_with_retry(
    video_id: str,
    start_time: int,
    end_time: int,
    max_attempts: int = 10,
) -> Optional[str]:
    last_result: Optional[str] = None

    for attempt in range(1, max_attempts + 1):
        polite_first = attempt > 2
        result_text = generate_questions_for_segment(
            video_id, start_time, end_time, polite_first=polite_first
        )

        if result_text:
            try:
                parsed = json.loads(result_text)
            except Exception:
                parsed = None

            if isinstance(parsed, dict) and 'error' in parsed:
                if not bool(parsed.get('error', {}).get('retryable')):
                    return result_text
            else:
                return result_text

        last_result = result_text
        time.sleep(random.uniform(1, 3))

    return last_result


def build_segments_from_duration(
    duration_seconds: int, interval_seconds: int, start_offset: int = 0
) -> List[Tuple[int, int]]:
    segments = []
    start = max(0, int(start_offset))
    step = max(1, int(interval_seconds))
    while start <= duration_seconds:
        end = min(start + step - 1, duration_seconds)
        segments.append((start, end))
        if end >= duration_seconds:
            break
        start = end + 1
    return segments


def _maybe_parse_json(text: Optional[str]):
    if text is None:
        return None
    if isinstance(text, (dict, list)):
        return text
    if not isinstance(text, str):
        return text
    cleaned = text.strip()
    if cleaned.startswith('```'):
        cleaned = cleaned[3:].lstrip()
        if cleaned.lower().startswith('json'):
            cleaned = cleaned[4:].lstrip()
        if cleaned.endswith('```'):
            cleaned = cleaned[:-3].rstrip()
    try:
        return json.loads(cleaned)
    except Exception:
        return text
