# Piggyback Learning

Django 6.0 application for downloading YouTube videos, extracting frames, and generating
educational comprehension questions for children ages 6–8. The app includes admin
processing tools, expert review workflows, and a kids-friendly playback/quiz interface.
It uses Daphne/Channels for ASGI with WebSocket support, Django REST Framework for APIs,
and drf-spectacular for API docs.

## Features

- YouTube downloads via `yt-dlp` (prefers 720p H.264 MP4 on first attempt)
- English subtitles (auto + manual when available) and metadata capture
- Frame extraction at 1 FPS with CSV/JSON manifests
- AI question generation (Google Gemini multimodal) with WebSocket progress updates
- Answer grading via RapidFuzz fuzzy matching
- Speech transcription (OpenAI Whisper) and TTS (OpenAI)
- Expert review and final question curation
- Kids library and quiz player UI

## Quickstart

1. Install system dependencies (FFmpeg required, Node.js optional). See the OS-specific section below.
2. Create and activate a virtual environment:
  ```bash
  python -m venv venv
  # Windows (PowerShell)
  .\venv\Scripts\Activate.ps1
  # Windows (cmd.exe)
  venv\Scripts\activate
  # macOS/Linux
  source venv/bin/activate
  ```
  If PowerShell blocks activation, run:
  ```bash
  Set-ExecutionPolicy -Scope Process Bypass
  ```
3. Install Python dependencies:
  ```bash
  python -m pip install --upgrade pip
  python -m pip install -r requirements.txt
  ```
4. Create `.env` in the project root (see Configuration below).
5. Run database migrations:
  ```bash
  python manage.py migrate
  ```
6. Start the dev server:
  ```bash
  python manage.py runserver 
  ```
7. Open:
  ```
  http://localhost:8000
  ```

## System Dependencies (Windows / macOS / Linux)

FFmpeg is required for muxing/format handling. Node.js LTS is optional but improves
`yt-dlp` reliability.

Verify after install:
```bash
ffmpeg -version
node -v
```

Windows:
- Install FFmpeg and ensure `ffmpeg.exe` is on your PATH (via winget/chocolatey or a static build).
- Install Node.js LTS (optional) and ensure `node` is on PATH.

macOS (Homebrew):
```bash
brew install ffmpeg
brew install node
```

Linux:
- Ubuntu/Debian:
  ```bash
  sudo apt-get update
  sudo apt-get install ffmpeg nodejs npm
  ```
- Fedora:
  ```bash
  sudo dnf install ffmpeg nodejs
  ```
- Arch:
  ```bash
  sudo pacman -S ffmpeg nodejs npm
  ```

## Configuration

Create a `.env` file in the project root:

```bash
GEMINI_API_KEY="your_gemini_key"         # Required for AI question generation (Google Gemini)
OPENAI_API_KEY="your_openai_key"         # Required for speech transcription and TTS

# Defaults are admin123 / expert123 if not set
ADMIN_PASSWORD="admin123"
EXPERT_PASSWORD="expert123"

# Optional: override the default Gemini model
GEMINI_MODEL="gemini-2.5-flash-lite"

# Optional: use a Netscape-format cookies file for restricted videos
YTDLP_COOKIEFILE="/path/to/cookies.txt"
# Alternate env var name also supported
YTDLP_COOKIES_FILE="/path/to/cookies.txt"
```

Notes:
- `.env` is loaded via `python-dotenv` in `core/settings.py`.
- Keep secrets out of git (add `.env` to `.gitignore`).

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

## API Endpoints (Core)

All API endpoints are under `/api/`. Interactive docs available at `/api/docs/`.

- `POST /api/verify-password` — role-based password check
- `POST /api/download` — download a YouTube video
- `POST /api/frames/<video_id>` — extract frames from a video
- `GET  /api/admin/videos` — list videos with processing status
- `POST /api/submit-questions` — submit generated questions
- `POST /api/check_answer` — grade a student answer
- `POST /api/transcribe` — speech-to-text via Whisper
- `POST /api/tts` — text-to-speech
- `GET  /api/config` — client config/thresholds
- `GET  /api/kids_videos` — videos with final question sets
- `GET  /api/final-questions/<video_id>` — curated questions for kids
- `GET  /api/videos-list` — all videos
- `GET  /api/expert-questions/<video_id>` — expert questions for a video
- `POST /api/expert-annotations` — save expert annotations
- `POST /api/expert-questions` — save expert questions
- `POST /api/save-final-questions` — save final curated questions
- `WS   /ws/questions/<video_id>` — stream question generation progress

## Project Structure

```
manage.py                 # Django management script
core/                     # Project config: settings.py, urls.py, asgi.py
ai/                       # Answer grading, transcription, TTS, config
pages/                    # Server-rendered HTML views (home, admin, children, expert)
videos/                   # Video, VideoAsset, ExtractedFrame models & services
quizgen/                  # Question generation pipeline, WebSocket consumer
review/                   # Expert review workflow & final question curation
user/                     # Password verification for admin/expert roles
templates/                # Django templates
public/assets/            # Static frontend assets (CSS, JS, images)
downloads/                # Runtime dir for videos, frames, metadata
db.sqlite3                # SQLite database (development)
requirements.txt
```

## Usage

### 1. Start the Application

```bash
# Make sure your virtual environment is activated
python manage.py runserver
```

Django admin is available at `http://localhost:8000/django-admin/`.

### 2. Access the Web Interface

Open your browser and go to: `http://localhost:8000`

### 3. Download a Video

1. Enter a YouTube URL in the form
2. Click "Download" to download the video and subtitles
3. The video will be saved in the `downloads/` directory

### 4. Extract Frames

1. After downloading, click "Extract Frames"
2. This will extract one frame per second from the video
3. Frames are saved as JPEG files with metadata in CSV/JSON format

### 5. Generate Questions

1. After extracting frames, click "Generate Questions"
2. Configure the time range and interval
3. Choose between single interval or full duration processing
4. Use "Stream Results" for real-time WebSocket progress updates

## Running Tests

```bash
# Run all tests
python manage.py test

# Run tests for a single app
python manage.py test ai

# Run a specific test
python manage.py test ai.tests.TestClassName.test_method
```

## Troubleshooting

- 403 Forbidden on download:
  - Try again (the downloader cycles player clients automatically).
  - Add a cookies file via `YTDLP_COOKIEFILE`.
  - Install Node.js LTS to improve extraction reliability.
- Low quality:
  - The first attempt prefers 720p H.264 MP4. If you need higher, adjust the
    format selector in `videos/services/download.py`.
- FFmpeg not found:
  - Install FFmpeg and ensure it is on your PATH.

### Common Issues

1. **"No module named 'cv2'"**

   ```bash
   pip install opencv-python
   ```

2. **"FFmpeg not found"**
   - On Windows: Download FFmpeg and add to PATH
   - On macOS: `brew install ffmpeg`
   - On Ubuntu: `sudo apt install ffmpeg`

3. **WebSocket connection issues**
   - Check firewall settings
   - Ensure the server is running on the correct port
   - Make sure Daphne is serving the ASGI app (the default `runserver` uses Daphne when installed)

### Performance Tips

- For long videos, use smaller intervals (30-60 seconds) to avoid API timeouts
- Frame extraction can take several minutes for long videos

## License

This project is for educational purposes. Please respect YouTube's Terms of Service and copyright laws when downloading content.

## Support

For issues related to:

- **yt-dlp**: Check [yt-dlp documentation](https://github.com/yt-dlp/yt-dlp)
- **Django**: Check [Django documentation](https://docs.djangoproject.com/en/6.0/)
- **Django REST Framework**: Check [DRF documentation](https://www.django-rest-framework.org/)
- **Django Channels**: Check [Channels documentation](https://channels.readthedocs.io/)

## Collaborators

- Ayush Gupta
- Riju Pant
- William Yang
- Adam Marx
- Shiven Patel
