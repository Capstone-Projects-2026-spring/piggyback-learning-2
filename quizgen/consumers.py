import csv
import json
from pathlib import Path
from typing import Any, Dict, Optional

from asgiref.sync import sync_to_async
from channels.generic.websocket import AsyncJsonWebsocketConsumer
from django.conf import settings

from quizgen.services.generation import generate_questions_for_segment_with_retry


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


def build_segments_from_duration(
    duration_seconds: int, interval_seconds: int, start_offset: int = 0
):
    """
    Inclusive segments: (0, 59), (60, 119) style or (0,60),(61,120) style?
    Your FastAPI code does (start, start+interval-1) then next start=end+1.
    """
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


def _segment_frame_debug(video_id: str, start: int, end: int) -> Dict[str, Any]:
    """
    Ported from admin_routes.py: inspects extracted frame coverage for segment.
    (Reads frame_data.csv just like FastAPI.)
    """
    downloads_dir: Path = settings.DOWNLOADS_DIR
    frames_dir = downloads_dir / video_id / 'extracted_frames'
    csv_path = frames_dir / 'frame_data.csv'
    debug: Dict[str, Any] = {'video_id': video_id, 'start': start, 'end': end}

    if not frames_dir.exists():
        debug['reason'] = 'frames_dir_missing'
        return debug
    if not csv_path.exists():
        debug['reason'] = 'frame_data_csv_missing'
        return debug

    min_ts = None
    max_ts = None
    total_rows = 0
    in_range = 0
    missing_files = 0

    try:
        with csv_path.open('r', encoding='utf-8') as handle:
            reader = csv.DictReader(handle)
            for row in reader:
                total_rows += 1
                ts_raw = (
                    row.get('Timestamp')
                    or row.get('Time_Seconds')
                    or row.get('Time_Formatted')
                )
                try:
                    if ts_raw is None:
                        continue
                    if isinstance(ts_raw, str) and ':' in ts_raw:
                        parts = [int(p) for p in ts_raw.split(':') if p.strip()]
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
                    filename = row.get('Filename') or ''
                    if filename and not (frames_dir / filename).exists():
                        missing_files += 1

        debug.update(
            {
                'total_frames': total_rows,
                'frames_in_range': in_range,
                'min_timestamp': min_ts,
                'max_timestamp': max_ts,
                'missing_frame_files': missing_files,
            }
        )

        if in_range == 0:
            debug['reason'] = 'no_frames_in_range'
        elif missing_files > 0:
            debug['reason'] = 'missing_frame_files'
        else:
            debug['reason'] = 'frames_present'

    except Exception as exc:
        debug['reason'] = 'csv_parse_error'
        debug['error'] = str(exc)

    return debug


def _wrap_segment_result(
    video_id: str, start: int, end: int, result_text: Optional[str], result_obj: Any
) -> Any:
    """
    Ported from admin_routes.py:
    Normalize failed generations into a structured error payload.
    """
    if isinstance(result_obj, dict) and 'error' in result_obj:
        return result_obj

    error_info: Optional[Dict[str, Any]] = None

    if result_text is None:
        error_info = {'reason': 'generation_returned_none'}
    elif isinstance(result_obj, str):
        error_info = {'reason': 'invalid_json', 'raw_preview': result_obj[:300]}
    elif isinstance(result_obj, dict) and 'questions' not in result_obj:
        error_info = {'reason': 'missing_questions', 'keys': list(result_obj.keys())}

    if error_info:
        frame_debug = _segment_frame_debug(video_id, start, end)
        return {'error': error_info, 'frame_debug': frame_debug}

    return result_obj


class QuestionsConsumer(AsyncJsonWebsocketConsumer):
    """
    FastAPI equivalent:
      /ws/questions/{video_id}

    Client sends once after connect:
      { start_seconds, interval_seconds, full_duration }

    Server streams:
      status -> segment_result(s) -> done
    """

    async def connect(self):
        self.video_id = self.scope['url_route']['kwargs']['video_id']
        await self.accept()

    async def receive_json(self, content, **kwargs):
        video_id: str = self.video_id

        try:
            start_seconds = int(content.get('start_seconds', 0))
            interval_seconds = int(content.get('interval_seconds', 60))
            full_duration = bool(content.get('full_duration', False))

            downloads_dir: Path = settings.DOWNLOADS_DIR
            frames_dir = downloads_dir / video_id / 'extracted_frames'
            if not frames_dir.exists():
                await self.send_json(
                    {
                        'type': 'error',
                        'message': 'Frames not found. Please extract frames first.',
                    }
                )
                await self.close()
                return

            # Determine duration (FastAPI uses frame_data.json)
            duration_seconds = None
            json_path = frames_dir / 'frame_data.json'
            if json_path.exists():
                try:
                    info = json.loads(json_path.read_text(encoding='utf-8'))
                    duration_seconds = int(
                        float(info.get('video_info', {}).get('duration_seconds', 0))
                    )
                except Exception:
                    duration_seconds = None

            # One-shot interval
            if not full_duration:
                start = max(0, int(start_seconds))
                end = start + max(1, int(interval_seconds)) - 1
                if (
                    duration_seconds is not None
                    and duration_seconds > 0
                    and end > duration_seconds
                ):
                    end = duration_seconds

                await self.send_json(
                    {
                        'type': 'status',
                        'message': f'Generating questions for {start}-{end}s...',
                    }
                )

                # Run blocking generation in threadpool
                result_text = await sync_to_async(
                    generate_questions_for_segment_with_retry, thread_sensitive=False
                )(video_id, start, end)

                result_obj = _maybe_parse_json(result_text)
                wrapped = _wrap_segment_result(
                    video_id, start, end, result_text, result_obj
                )

                await self.send_json(
                    {
                        'type': 'segment_result',
                        'start': start,
                        'end': end,
                        'result': wrapped,
                    }
                )
                await self.send_json({'type': 'done', 'auto_saved': False})
                await self.close()
                return

            # Full-duration loop
            if duration_seconds is None or duration_seconds <= 0:
                await self.send_json(
                    {'type': 'error', 'message': 'Unable to determine video duration.'}
                )
                await self.close()
                return

            segments = build_segments_from_duration(
                duration_seconds, interval_seconds, start_seconds
            )

            await self.send_json(
                {
                    'type': 'status',
                    'message': f'Starting full-duration generation over {len(segments)} segments.',
                }
            )

            aggregated = {
                'video_id': video_id,
                'interval_seconds': int(interval_seconds),
                'start_offset': int(start_seconds),
                'duration_seconds': int(duration_seconds),
                'segments': [],
            }

            for idx, (seg_start, seg_end) in enumerate(segments, start=1):
                await self.send_json(
                    {
                        'type': 'status',
                        'message': f'[{idx}/{len(segments)}] {seg_start}-{seg_end}s',
                    }
                )

                result_text = await sync_to_async(
                    generate_questions_for_segment_with_retry, thread_sensitive=False
                )(video_id, seg_start, seg_end)

                result_obj = _maybe_parse_json(result_text)
                wrapped = _wrap_segment_result(
                    video_id, seg_start, seg_end, result_text, result_obj
                )

                aggregated['segments'].append(
                    {'start': seg_start, 'end': seg_end, 'result': wrapped}
                )

                await self.send_json(
                    {
                        'type': 'segment_result',
                        'start': seg_start,
                        'end': seg_end,
                        'result': wrapped,
                    }
                )

            await self.send_json(
                {
                    'type': 'done',
                    'segments_count': len(segments),
                    'auto_saved': False,
                    'data': aggregated,
                }
            )
            await self.close()

        except Exception as e:
            try:
                await self.send_json({'type': 'error', 'message': str(e)})
                await self.close()
            except Exception:
                pass
