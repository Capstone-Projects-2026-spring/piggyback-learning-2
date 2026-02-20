from pathlib import Path
from typing import Dict, List, Optional

from app.settings import DOWNLOADS_DIR
from app.settings import VIDEO_EXTENSIONS


def find_primary_video_file(video_dir: Path) -> Optional[Path]:
    if not video_dir.exists() or not video_dir.is_dir():
        return None
    for ext in VIDEO_EXTENSIONS:
        matches = sorted(video_dir.glob(f"*{ext}"))
        if matches:
            return matches[0]
    return None


def list_question_json_files() -> List[Dict[str, str]]:
    files: List[Dict[str, str]] = []
    if not DOWNLOADS_DIR.exists():
        return files
    for video_dir in sorted(DOWNLOADS_DIR.iterdir()):
        if not video_dir.is_dir():
            continue
        questions_dir = video_dir / "questions"
        if not questions_dir.is_dir():
            continue
        for json_file in sorted(questions_dir.glob("*.json")):
            try:
                rel_path = json_file.relative_to(DOWNLOADS_DIR).as_posix()
            except ValueError:
                continue
            files.append(
                {
                    "video_id": video_dir.name,
                    "name": json_file.name,
                    "rel_path": rel_path,
                }
            )
    files.sort(key=lambda item: (item["video_id"], item["name"]))
    return files
