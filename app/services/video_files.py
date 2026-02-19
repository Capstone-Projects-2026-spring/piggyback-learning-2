from pathlib import Path
from typing import Optional

from app.settings import VIDEO_EXTENSIONS


def find_primary_video_file(video_dir: Path) -> Optional[Path]:
    if not video_dir.exists() or not video_dir.is_dir():
        return None
    for ext in VIDEO_EXTENSIONS:
        matches = sorted(video_dir.glob(f"*{ext}"))
        if matches:
            return matches[0]
    return None
