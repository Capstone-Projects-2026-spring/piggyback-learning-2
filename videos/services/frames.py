import json
from pathlib import Path
from typing import Any, Dict, List

import cv2
from django.conf import settings
from django.db import transaction

from videos.models import ExtractedFrame, Video, VideoAsset
from videos.services.download import find_primary_video_file


def extract_frames_per_second_for_video(video_id: str) -> Dict[str, Any]:
    """
    Port of main.py:extract_frames_per_second_for_video
    Still writes extracted_frames/ images for your UI, but DB is the canonical store.
    """
    downloads_dir: Path = settings.DOWNLOADS_DIR
    folder_path = downloads_dir / video_id
    if not folder_path.exists():
        return {
            'success': False,
            'message': f"Folder '{video_id}' not found.",
            'files': [],
        }

    video_file = find_primary_video_file(folder_path)
    if not video_file:
        return {
            'success': False,
            'message': f"No video files found in '{video_id}'.",
            'files': [],
        }

    output_dir = folder_path / 'extracted_frames'
    output_dir.mkdir(exist_ok=True)

    cap = cv2.VideoCapture(str(video_file))
    if not cap.isOpened():
        return {
            'success': False,
            'message': f'Error opening video file: {video_file.name}',
            'files': [],
        }

    fps = cap.get(cv2.CAP_PROP_FPS)
    total_video_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    if fps is None or fps <= 0:
        cap.release()
        return {'success': False, 'message': 'Invalid FPS detected.', 'files': []}

    duration = total_video_frames / fps
    total_seconds = int(duration)

    frame_rows: List[ExtractedFrame] = []
    frame_data_json: List[Dict[str, Any]] = []

    for second in range(total_seconds):
        frame_number = int(second * fps)
        cap.set(cv2.CAP_PROP_POS_FRAMES, frame_number)
        ret, frame = cap.read()
        if not ret:
            continue

        frame_filename = f'frame_{second:04d}s.jpg'
        frame_path = output_dir / frame_filename
        cv2.imwrite(str(frame_path), frame)

        frame_data_json.append(
            {
                'frame_number': second + 1,
                'timestamp_seconds': second,
                'timestamp_formatted': f'{second // 60:02d}:{second % 60:02d}',
                'filename': frame_filename,
                'file_path': str(frame_path),
            }
        )

    cap.release()

    # compatibility files (optional but keeps your existing tooling/UI happy)
    json_path = output_dir / 'frame_data.json'
    json_path.write_text(
        json.dumps(
            {
                'video_info': {
                    'filename': video_file.name,
                    'duration_seconds': duration,
                    'total_frames': total_video_frames,
                    'fps': fps,
                    'extracted_frames': len(frame_data_json),
                },
                'frames': frame_data_json,
            },
            indent=2,
            ensure_ascii=False,
        ),
        encoding='utf-8',
    )

    csv_path = output_dir / 'frame_data.csv'
    with csv_path.open('w', encoding='utf-8') as f:
        f.write('Frame,Timestamp,Time_Formatted,Filename\n')
        for fr in frame_data_json:
            f.write(
                f'{fr["frame_number"]},{fr["timestamp_seconds"]},{fr["timestamp_formatted"]},{fr["filename"]}\n'
            )

    # ---- DB sync ----
    with transaction.atomic():
        video, _ = Video.objects.get_or_create(id=video_id)
        # wipe old frames (idempotent)
        ExtractedFrame.objects.filter(video=video).delete()

        # insert frames
        to_create: List[ExtractedFrame] = []
        for fr in frame_data_json:
            rel_path = (
                (output_dir / fr['filename']).relative_to(downloads_dir).as_posix()
            )
            to_create.append(
                ExtractedFrame(
                    video=video,
                    frame_number=int(fr['frame_number']),
                    timestamp_seconds=int(fr['timestamp_seconds']),
                    timestamp_formatted=str(fr['timestamp_formatted']),
                    filename=str(fr['filename']),
                    file_path=f'downloads/{rel_path}',
                    subtitle_text='',
                )
            )
        ExtractedFrame.objects.bulk_create(to_create, batch_size=500)

        # record assets too (optional)
        # keep existing video assets, but refresh frame assets
        VideoAsset.objects.filter(
            video=video,
            kind='other',
            file_path__contains=f'downloads/{video_id}/extracted_frames/',
        ).delete()
        for fr in frame_data_json:
            rel = (output_dir / fr['filename']).relative_to(downloads_dir).as_posix()
            VideoAsset.objects.get_or_create(
                video=video, file_path=f'downloads/{rel}', defaults={'kind': 'other'}
            )
        # add csv/json
        VideoAsset.objects.get_or_create(
            video=video,
            file_path=f'downloads/{(json_path.relative_to(downloads_dir)).as_posix()}',
            defaults={'kind': 'other'},
        )
        VideoAsset.objects.get_or_create(
            video=video,
            file_path=f'downloads/{(csv_path.relative_to(downloads_dir)).as_posix()}',
            defaults={'kind': 'other'},
        )

    links: List[str] = []
    for p in sorted(output_dir.iterdir()):
        if p.is_file():
            rel = p.relative_to(downloads_dir).as_posix()
            links.append(f'/downloads/{rel}')

    return {
        'success': True,
        'message': f"Extracted {len(frame_data_json)} frames to '{output_dir.name}'.",
        'files': links,
        'video_id': video_id,
        'output_dir': f'/downloads/{video_id}/extracted_frames',
        'count': len(frame_data_json),
    }
