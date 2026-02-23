import base64
import io
import json
import random
import time
from typing import Any, Dict, List, Optional, Tuple

from django.conf import settings
from django.db import transaction
from openai import OpenAI
from PIL import Image

from quizgen.models import GeneratedQuestion, Segment, SegmentLLMResult
from videos.models import ExtractedFrame, Video


def get_openai_client() -> OpenAI:
    return OpenAI()


def encode_image_to_base64(abs_image_path: str, max_size=(512, 512)) -> Optional[str]:
    try:
        with Image.open(abs_image_path) as img:
            if img.mode != 'RGB':
                img = img.convert('RGB')
            img.thumbnail(max_size, Image.Resampling.LANCZOS)
            buffer = io.BytesIO()
            img.save(buffer, format='JPEG', quality=85)
            return base64.b64encode(buffer.getvalue()).decode('utf-8')
    except Exception:
        return None


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


def _build_transcript_from_frames(frames: List[ExtractedFrame]) -> str:
    parts = []
    for fr in frames:
        txt = (fr.subtitle_text or '').strip()
        if txt:
            parts.append(f'[{fr.timestamp_formatted or fr.timestamp_seconds}] {txt}')
    return (
        '\n'.join(parts) if parts else 'No transcript available for this video segment.'
    )


def _sample_frames(
    frames: List[ExtractedFrame], max_frames: int = 5
) -> List[ExtractedFrame]:
    if len(frames) <= max_frames:
        return frames
    step = max(1, len(frames) // max_frames)
    sampled = frames[0::step][:max_frames]
    return sampled


def generate_questions_for_segment(
    client: OpenAI,
    video_id: str,
    start_time: int,
    end_time: int,
    polite_first: bool = False,
    model: str = 'gpt-4o',
) -> str:
    """
    DB-backed version of main.py:generate_questions_for_segment
    Uses ExtractedFrame rows instead of CSV.
    """
    video = Video.objects.get(id=video_id)

    # frames in window inclusive
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
I am providing you with sequential frames from a {duration}-second segment ({start_time}s to {end_time}s) of a video,
along with the complete transcript above.

1. Provide ONE short, child-friendly comprehension question for EACH of the following categories:
   - Character
   - Setting
   - Feeling
   - Action
   - Causal Relationship
   - Outcome
   - Prediction

2. Rank the questions based on how relevant and good it is to test comprehension and active viewing; best question = rank 1

3. Return JSON only (no extra text) in this structure:
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

I am providing you with sequential frames from a {duration}-second segment ({start_time}s to {end_time}s) of this educational children's video.

Please create ONE short, child-friendly comprehension question for EACH category:
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

    image_content: List[Dict[str, Any]] = []
    successful = 0
    for fr in sampled:
        # fr.file_path is like "downloads/<...>" (relative), build absolute
        rel = fr.file_path.replace('downloads/', '', 1)
        abs_path = str(settings.DOWNLOADS_DIR / rel)
        b64 = encode_image_to_base64(abs_path)
        if b64:
            image_content.append(
                {
                    'type': 'image_url',
                    'image_url': {
                        'url': f'data:image/jpeg;base64,{b64}',
                        'detail': 'low',
                    },
                }
            )
            successful += 1

    if successful == 0:
        return json.dumps(
            {'error': {'reason': 'frame_encoding_failed', 'retryable': False}}
        )

    prompt_sequence = [('standard', base_prompt), ('polite', polite_prompt)]
    if polite_first:
        prompt_sequence = [('polite', polite_prompt), ('standard', base_prompt)]

    last_error_payload: Optional[Dict[str, Any]] = None

    def _call_llm(content_with_prompt: List[Dict[str, Any]]) -> Optional[str]:
        nonlocal last_error_payload
        max_retries = 3
        for attempt in range(max_retries):
            try:
                resp = client.chat.completions.create(
                    model=model,
                    messages=[
                        {'role': 'system', 'content': system_message},
                        {'role': 'user', 'content': content_with_prompt},
                    ],
                    max_tokens=1500,
                    temperature=0.3,
                    response_format={'type': 'json_object'},
                )
                finish_reason = resp.choices[0].finish_reason
                result_content = resp.choices[0].message.content
                if finish_reason == 'content_filter':
                    last_error_payload = {'reason': 'model_refusal', 'retryable': False}
                    return None
                if result_content:
                    return result_content
                last_error_payload = {'reason': 'empty_response', 'retryable': True}
            except Exception as e:
                msg = str(e)
                if 'rate_limit_exceeded' in msg and attempt < max_retries - 1:
                    wait_time = (2**attempt) + random.uniform(0, 1)
                    time.sleep(wait_time)
                    last_error_payload = {
                        'reason': 'rate_limit_exceeded',
                        'retryable': True,
                        'message': msg,
                    }
                    continue
                last_error_payload = {
                    'reason': 'openai_error',
                    'retryable': True,
                    'message': msg,
                }
                return None
        return None

    tried_transcript_only = False

    for attempt_round, (label, prompt) in enumerate(prompt_sequence):
        content_with_prompt = [{'type': 'text', 'text': prompt}] + image_content
        result_content = _call_llm(content_with_prompt)
        if result_content:
            return result_content

        if (
            not tried_transcript_only
            and last_error_payload
            and last_error_payload.get('reason') in {'model_refusal', 'empty_response'}
        ):
            tried_transcript_only = True
            transcript_only_prompt = (
                prompt
                + '\n\nIf visuals are unavailable, answer using the transcript only.'
            )
            result_content = _call_llm(
                [{'type': 'text', 'text': transcript_only_prompt}]
            )
            if result_content:
                return result_content

        if attempt_round == 0 and len(prompt_sequence) > 1:
            continue

    if last_error_payload is None:
        last_error_payload = {'reason': 'generation_failed', 'retryable': True}
    return json.dumps({'error': last_error_payload})


def generate_questions_for_segment_with_retry(
    client: OpenAI,
    video_id: str,
    start_time: int,
    end_time: int,
    max_attempts: int = 10,
    model: str = 'gpt-4o',
) -> str:
    last_result: Optional[str] = None
    for attempt in range(1, max_attempts + 1):
        polite_first = attempt > 2
        if attempt > 1:
            time.sleep(random.uniform(1, 3))

        result_text = generate_questions_for_segment(
            client=client,
            video_id=video_id,
            start_time=start_time,
            end_time=end_time,
            polite_first=polite_first,
            model=model,
        )
        last_result = result_text

        try:
            parsed = json.loads(result_text)
        except Exception:
            # if it isn't JSON, treat as bad and retry
            continue

        if isinstance(parsed, dict) and 'error' in parsed:
            retryable = bool(parsed.get('error', {}).get('retryable'))
            if not retryable:
                return result_text
            continue

        return result_text

    return last_result or json.dumps(
        {'error': {'reason': 'generation_failed', 'retryable': True}}
    )


def persist_segment_result(
    video_id: str, start: int, end: int, result_text: str
) -> Dict[str, Any]:
    """
    Writes Segment, SegmentLLMResult, GeneratedQuestion rows.
    Returns a normalized dict payload.
    """
    video = Video.objects.get(id=video_id)

    try:
        parsed = json.loads(result_text)
    except Exception:
        parsed = {
            'error': {
                'reason': 'invalid_json',
                'retryable': True,
                'raw_preview': result_text[:300],
            }
        }

    with transaction.atomic():
        segment, _ = Segment.objects.get_or_create(
            video=video, start_seconds=start, end_seconds=end
        )

        # clear generated questions for this segment before rewrite
        GeneratedQuestion.objects.filter(segment=segment).delete()

        has_error = isinstance(parsed, dict) and 'error' in parsed
        error_payload = parsed.get('error', {}) if has_error else {}

        best_question = ''
        questions = {}
        if isinstance(parsed, dict):
            best_question = str(parsed.get('best_question') or '')
            questions = parsed.get('questions') or {}

        llm_result, _ = SegmentLLMResult.objects.update_or_create(
            segment=segment,
            defaults={
                'raw': parsed if isinstance(parsed, dict) else {'raw': parsed},
                'best_question': best_question,
                'has_error': bool(has_error),
                'error': error_payload
                if isinstance(error_payload, dict)
                else {'error': error_payload},
            },
        )

        if isinstance(questions, dict) and not has_error:
            for qtype, info in questions.items():
                if not isinstance(info, dict):
                    continue
                GeneratedQuestion.objects.create(
                    segment=segment,
                    qtype=str(qtype),
                    question=str(info.get('q') or ''),
                    answer=str(info.get('a') or ''),
                    llm_rank=(
                        int(info['rank'])
                        if str(info.get('rank') or '').strip().isdigit()
                        else None
                    ),
                )

    return {
        'start': start,
        'end': end,
        'result': parsed,
        'segment_id': segment.id,
        'has_error': has_error,
    }


def generate_for_video(
    video_id: str,
    interval_seconds: int = 60,
    start_offset_seconds: int = 0,
    full_duration: bool = True,
    duration_seconds: Optional[int] = None,
    max_attempts: int = 10,
    model: str = 'gpt-4o',
) -> Dict[str, Any]:
    video = Video.objects.get(id=video_id)

    # Decide duration
    dur = duration_seconds or video.duration_seconds
    if not dur:
        # fallback: infer duration from last frame timestamp
        last_frame = (
            ExtractedFrame.objects.filter(video=video)
            .order_by('-timestamp_seconds')
            .first()
        )
        dur = last_frame.timestamp_seconds if last_frame else 0

    segments = build_segments_from_duration(
        int(dur), int(interval_seconds), int(start_offset_seconds)
    )

    client = get_openai_client()

    results = []
    for start, end in segments:
        raw = generate_questions_for_segment_with_retry(
            client=client,
            video_id=video_id,
            start_time=start,
            end_time=end,
            max_attempts=max_attempts,
            model=model,
        )
        results.append(persist_segment_result(video_id, start, end, raw))

    return {
        'success': True,
        'video_id': video_id,
        'count': len(results),
        'segments': results,
    }
