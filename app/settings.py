from pathlib import Path
import os

from dotenv import load_dotenv

# Load env vars from .env (if present) and .env.txt (explicit file requested by user)
load_dotenv()
load_dotenv(".env.txt")

BASE_DIR = Path(__file__).resolve().parent.parent
DOWNLOADS_DIR = BASE_DIR / "downloads"
TEMPLATES_DIR = BASE_DIR / "templates"
PUBLIC_ASSETS_DIR = BASE_DIR / "public" / "assets"
DOWNLOADS_DIR.mkdir(parents=True, exist_ok=True)

ADMIN_PASSWORD = os.getenv("ADMIN_PASSWORD", "admin123")
EXPERT_PASSWORD = os.getenv("EXPERT_PASSWORD", "expert123")
GEMINI_API_KEY = os.getenv("GEMINI_API_KEY", "").strip()
QUESTION_PROVIDER_DEFAULT = os.getenv("QUESTION_PROVIDER_DEFAULT", "openai").strip().lower()

VIDEO_EXTENSIONS = (".mp4",".webm",".mkv",".mov")

EXPERT_QUESTION_TYPES = [
    ("character","Character"),
    ("settings","Setting"),
    ("feeling","Feeling"),
    ("action","Action"),
    ("causal","Causal"),
    ("outcome","Outcome"),
    ("prediction","Prediction"),    
]

EXPERT_QUESTION_TYPE_VALUES = {value for value, _ in EXPERT_QUESTION_TYPES}
EXPERT_QUESTION_TYPE_LABELS = {value: label for value, label in EXPERT_QUESTION_TYPES}
