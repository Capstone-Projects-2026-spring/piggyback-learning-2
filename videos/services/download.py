import json
import os
import re
import shutil
from pathlib import Path
from typing import Any, Dict, List, Optional

import yt_dlp
from django.conf import settings
from django.db import transaction

from videos.models import Video, VideoAsset

VIDEO_EXTENSIONS = ('.mp4', '.webm', '.mkv', '.mov')


def _looks_like_mp4(path: Path) -> bool:
    try:
        with path.open('rb') as handle:
            header = handle.read(64)
        return b'ftyp' in header
    except Exception:
        return False


def find_primary_video_file(video_dir: Path) -> Optional[Path]:
    if not video_dir.exists() or not video_dir.is_dir():
        return None
    for ext in VIDEO_EXTENSIONS:
        matches = sorted(video_dir.glob(f'*{ext}'))
        if matches:
            return matches[0]
    return None


def download_youtube(url: str) -> Dict[str, Any]:
    """
    Port of main.py:download_youtube.
    Downloads into downloads/<video_id>/ and upserts Video + VideoAsset rows.
    """
    result = {
        'success': False,
        'message': 'Download error',
        'video_id': None,
        'title': None,
        'thumbnail': None,
        'files': [],
    }

    if not url:
        result['message'] = 'No URL provided.'
        return result

    if not (url.startswith('http') and ('youtube.com' in url or 'youtu.be' in url)):
        result['message'] = 'Please provide a valid YouTube URL.'
        return result

    downloads_dir: Path = settings.DOWNLOADS_DIR

    try:
        # Step 1: metadata only
        with yt_dlp.YoutubeDL({'quiet': True, 'no_warnings': True}) as ydl:
            info = ydl.extract_info(url, download=False)
            video_id = info.get('id', 'unknown')
            title = info.get('title', 'Untitled Video')
            thumbnail = info.get('thumbnail', '')
            duration = info.get('duration', 0)

        result['video_id'] = video_id
        result['title'] = title
        result['thumbnail'] = thumbnail

        video_dir = downloads_dir / video_id
        video_dir.mkdir(parents=True, exist_ok=True)

        has_ffmpeg = shutil.which('ffmpeg') is not None
        has_node = shutil.which('node') is not None

        def _clean_ytdlp_error(message: str) -> str:
            return re.sub(r'\x1b\[[0-9;]*m', '', message).strip()

        ydl_opts: Dict[str, Any] = {
            'format': (
                'bv*[vcodec^=avc1][ext=mp4][height<=?720]+ba[acodec^=mp4a]/'
                'b[ext=mp4][height<=?720]/b[height<=?720]/b'
            ),
            'merge_output_format': 'mp4',
            'allow_unplayable_formats': True,
            'outtmpl': str(video_dir / f'{video_id}.%(ext)s'),
            'quiet': False,
            'no_warnings': True,
            'noprogress': True,
            'noplaylist': True,
            'writethumbnail': True,
            'writeinfojson': True,
            'prefer_ffmpeg': True,
            'postprocessors': [{'key': 'FFmpegVideoRemuxer', 'preferedformat': 'mp4'}],
        }

        if has_node:
            ydl_opts['js_runtimes'] = {'node': {}}
            ydl_opts['external_deps'] = {'ejs': 'github'}

        cookies_file = (
            os.getenv('YTDLP_COOKIEFILE') or os.getenv('YTDLP_COOKIES_FILE') or ''
        ).strip()
        if cookies_file:
            ydl_opts['cookiefile'] = cookies_file

        if not has_ffmpeg:
            ydl_opts['compat_opts'] = ['no-sabr']
            ydl_opts['format'] = 'best[ext=mp4]/best'

        def _download_with_format_fallback(opts: Dict[str, Any]) -> None:
            try:
                with yt_dlp.YoutubeDL(opts) as ydl:  # type: ignore
                    ydl.download([url])
            except yt_dlp.utils.DownloadError as e:  # type: ignore
                message = str(e)
                if 'Requested format is not available' in message:
                    fallback_opts = dict(opts)
                    if has_ffmpeg:
                        fallback_opts['format'] = 'bestvideo+bestaudio/best'
                        fallback_opts.pop('merge_output_format', None)
                    else:
                        fallback_opts['format'] = 'best'
                        fallback_opts.pop('merge_output_format', None)
                    fallback_opts.pop('postprocessors', None)
                    fallback_opts.pop('extractor_args', None)
                    with yt_dlp.YoutubeDL(fallback_opts) as ydl:  # type: ignore
                        ydl.download([url])
                else:
                    raise

        player_client_candidates = [
            ['android', 'web'],
            ['tv', 'web'],
            ['ios', 'web'],
            ['web'],
        ]

        last_error = None
        used_player_client = None
        for player_client in player_client_candidates:
            attempt_opts = dict(ydl_opts)
            attempt_opts['extractor_args'] = {
                'youtube': {'player_client': player_client}
            }
            try:
                _download_with_format_fallback(attempt_opts)
                last_error = None
                used_player_client = player_client
                break
            except yt_dlp.utils.DownloadError as e:  # type: ignore
                message = str(e)
                if 'HTTP Error 403' in message or '403' in message:
                    last_error = e
                    continue
                raise

        if last_error is not None:
            raise last_error

        subtitle_warning = None
        subtitle_opts: Dict[str, Any] = {
            'outtmpl': str(video_dir / f'{video_id}.%(ext)s'),
            'writesubtitles': True,
            'writeautomaticsub': True,
            'subtitleslangs': ['en'],
            'subtitlesformat': 'vtt',
            'skip_download': True,
            'quiet': True,
            'no_warnings': True,
            'noprogress': True,
            'noplaylist': True,
        }

        if has_node:
            subtitle_opts['js_runtimes'] = {'node': {}}
            subtitle_opts['external_deps'] = {'ejs': 'github'}
        if cookies_file:
            subtitle_opts['cookiefile'] = cookies_file
        if used_player_client:
            subtitle_opts['extractor_args'] = {
                'youtube': {'player_client': used_player_client}
            }

        try:
            with yt_dlp.YoutubeDL(subtitle_opts) as ydl:  # type: ignore
                ydl.download([url])
        except yt_dlp.utils.DownloadError as e:  # type: ignore
            subtitle_warning = _clean_ytdlp_error(str(e))

        created: List[str] = []
        for p in sorted(video_dir.iterdir()):
            if p.is_file():
                created.append(p.relative_to(downloads_dir).as_posix())

        video_path = find_primary_video_file(video_dir)
        if not video_path:
            result['message'] = 'Download completed but no video file was found.'
            result['files'] = created
            return result

        if video_path.suffix.lower() == '.mp4' and not _looks_like_mp4(video_path):
            result['message'] = (
                'Downloaded file is not a valid MP4 container (likely HLS/TS). '
                'Install ffmpeg or re-download with a non-SABR format.'
            )
            result['files'] = created
            return result

        local_path = f'/downloads/{video_path.relative_to(downloads_dir).as_posix()}'

        meta = {
            'video_id': video_id,
            'title': title,
            'thumbnail': thumbnail,
            'duration': duration,
            'local_path': local_path,
        }

        (video_dir / 'meta.json').write_text(
            json.dumps(meta, indent=2), encoding='utf-8'
        )

        # ---- DB upserts ----
        with transaction.atomic():
            video, _ = Video.objects.update_or_create(
                id=video_id,
                defaults={
                    'title': title or '',
                    'thumbnail_url': thumbnail or '',
                    'duration_seconds': int(duration) if duration else None,
                    'local_video_path': local_path,
                },
            )

            # refresh assets
            VideoAsset.objects.filter(video=video).delete()
            for rel in created:
                kind = 'other'
                lower = rel.lower()
                if lower.endswith(('.mp4', '.webm', '.mkv', '.mov')):
                    kind = 'video'
                elif lower.endswith('.vtt'):
                    kind = 'subtitle'
                elif lower.endswith(('.jpg', '.jpeg', '.png', '.webp')):
                    kind = 'thumbnail'
                elif lower.endswith('.json'):
                    kind = 'meta'
                VideoAsset.objects.create(
                    video=video, file_path=f'downloads/{rel}', kind=kind
                )

        result.update(
            {
                'success': True,
                'message': 'Video downloaded successfully.',
                'files': created,
                'duration': duration,
                'local_path': local_path,
            }
        )
        if subtitle_warning:
            result['subtitle_warning'] = subtitle_warning
        return result

    except yt_dlp.utils.DownloadError as e:  # type: ignore
        result['message'] = f'Download error: {e}'
        return result
    except Exception as e:
        result['message'] = f'Unexpected error: {e}'
        return result
