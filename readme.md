# Piggyback Learning - YouTube Downloader + AI Question Generation

A FastAPI web app that downloads YouTube videos, extracts frames, and generates
educational comprehension questions for kids. It includes an admin workflow for
processing, an expert review UI, and a kids-friendly playback/quiz interface.

## Features

- YouTube download via yt-dlp (prefers 720p H.264 MP4 on first attempt)
- English subtitles (auto + manual when available) and metadata capture
- Frame extraction at 1 FPS with CSV/JSON manifests
- AI question generation with WebSocket progress updates
- Expert review and final question curation
- Kids library and quiz player UI

## Prerequisites

- Python 3.8+
- OpenAI API key (question generation + grading)
- FFmpeg (recommended for muxing/format handling)
- Node.js LTS (optional, improves yt-dlp extraction reliability)
- Optional: YouTube cookies file for restricted videos

## Installation

1. Create and activate a virtual environment.
   ```bash
   python -m venv venv
   # Windows
   venv\Scripts\activate
   # macOS/Linux
   source venv/bin/activate
   ```
2. Install dependencies.
   ```bash
   pip install -r requirements.txt
   ```

## Configuration (.env or .env.txt)

Create a `.env` file in the project root:

```bash
OPENAI_API_KEY="your_openai_key"
ADMIN_PASSWORD="admin"
EXPERT_PASSWORD="expert"
# Optional: use a Netscape-format cookies file for restricted videos
YTDLP_COOKIEFILE="C:\\path\\to\\cookies.txt"
```

Notes:
- `.env` and `.env.txt` are both loaded if present.
- Keep secrets out of git (add `.env` to `.gitignore`).

## Optional: Node.js (recommended)

yt-dlp can use Node.js to handle modern YouTube challenges. Verify install:

```bash
node -v
```

## Run the App

```bash
uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

Open the home page in your browser:

```
http://localhost:8000
```

## App Flows

### Admin

1. From the home page, choose Admin and enter the admin password.
2. Use the Admin panel to download videos, extract frames, and generate questions.

### Expert Review

1. From the home page, choose Expert and enter the expert password.
2. Use the Expert Preview page to review or create questions.

### Kids

1. Go to the Kids page.
2. Browse videos and play quizzes.

## API Endpoints (core)

- `POST /api/verify-password`
- `POST /api/download`
- `POST /api/frames/{video_id}`
- `GET /api/admin/videos`
- `POST /api/submit-questions`
- `WS  /ws/questions/{video_id}`
- `GET /api/kids_videos`
- `GET /api/final-questions/{video_id}`
- `GET /api/videos-list`
- `GET /api/expert-questions/{video_id}`

## Project Structure

```
main.py
admin_routes.py
video_quiz_routes.py
templates/
  admin.html
  children.html
  expert_preview.html
  home.html
  video_quiz.html
downloads/
static/
requirements.txt
readme.md
```

## Troubleshooting

- 403 Forbidden on download:
  - Try again (the downloader cycles player clients automatically).
  - Add a cookies file via `YTDLP_COOKIEFILE`.
  - Install Node.js LTS to improve extraction reliability.
- Low quality:
  - The first attempt prefers 720p H.264 MP4. If you need higher, adjust the
    format selector in `main.py` inside `download_youtube()`.
- FFmpeg not found:
  - Install FFmpeg and ensure it is on your PATH.

## License

Educational use only. Respect YouTube's Terms of Service and copyright laws.
