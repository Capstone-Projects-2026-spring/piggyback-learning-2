# Piggyback Learning

FastAPI application for downloading YouTube videos, extracting frames, and generating
educational comprehension questions. The app includes admin processing tools, expert
review workflows, and a kids-friendly playback/quiz interface.

## Features

- YouTube downloads via `yt-dlp` (prefers 720p H.264 MP4 on first attempt)
- English subtitles (auto + manual when available) and metadata capture
- Frame extraction at 1 FPS with CSV/JSON manifests
- AI question generation with WebSocket progress updates
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
4. Create `.env` in the project root (defaults shown below).
5. Run the app:
  ```bash
  python -m uvicorn main:app --reload --host 0.0.0.0 --port 8000
  ```
6. Open:
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
OPENAI_API_KEY="your_openai_key"
# Defaults are admin123 / expert123 if not set
ADMIN_PASSWORD="admin123"
EXPERT_PASSWORD="expert123"
# Optional: use a Netscape-format cookies file for restricted videos
YTDLP_COOKIEFILE="C:\\path\\to\\cookies.txt"
# Alternate env var name also supported
YTDLP_COOKIES_FILE="C:\\path\\to\\cookies.txt"
```

Notes:
- `.env` and `.env.txt` are both loaded if present.
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

<<<<<<< HEAD
## API Endpoints (Core)

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
=======
Then install with:

```bash
pip install -r requirements.txt
```

### 4. Set Up OpenAI API Key

You need an OpenAI API key to use the question generation features. Set it as an environment variable:

```bash
# Windows (Command Prompt)
set OPENAI_API_KEY=your_api_key_here

# Windows (PowerShell)
$env:OPENAI_API_KEY="your_api_key_here"

# macOS/Linux
export OPENAI_API_KEY="your_api_key_here"
```

Alternatively, you can enter the API key directly in the web interface when generating questions.

### 5. Create Required Directories

The application will create necessary directories automatically, but you can create the template directory structure:

```
your-project/
├── main.py
├── templates/
│   ├── download.html
│   ├── preview.html
│   ├── frames.html
│   └── questions.html
├── downloads/
└── venv/
```

## HTML Templates

You'll need to create the following HTML template files in the `templates/` directory. Here are basic examples:

### templates/download.html

```html
<!DOCTYPE html>
<html>
  <head>
    <title>YouTube Downloader</title>
  </head>
  <body>
    <h1>YouTube Video Downloader</h1>
    <form action="/download" method="post">
      <label for="url">YouTube URL:</label>
      <input type="text" id="url" name="url" required style="width: 400px;" />
      <button type="submit">Download</button>
    </form>
  </body>
</html>
```

### templates/preview.html

```html
<!DOCTYPE html>
<html>
  <head>
    <title>Download Result</title>
  </head>
  <body>
    <h1>Download Result</h1>
    {% if success %}
    <p style="color: green;">{{ message }}</p>
    <p>Video ID: {{ video_id }}</p>

    {% if current_video_url %}
    <video width="640" height="360" controls>
      <source src="{{ current_video_url }}" type="video/mp4" />
      {% if current_sub_url %}
      <track
        src="{{ current_sub_url }}"
        kind="subtitles"
        srclang="en"
        label="English"
      />
      {% endif %}
    </video>
    {% endif %}

    <p><a href="/frames/{{ video_id }}">Extract Frames</a></p>
    {% else %}
    <p style="color: red;">{{ message }}</p>
    {% endif %}

    <p><a href="/">Download Another Video</a></p>
  </body>
</html>
```

### templates/frames.html

```html
<!DOCTYPE html>
<html>
  <head>
    <title>Frame Extraction</title>
  </head>
  <body>
    <h1>Frame Extraction - {{ video_id }}</h1>

    {% if not ran %}
    <form method="post">
      <button type="submit">Extract Frames (1 per second)</button>
    </form>
    {% else %} {% if success %}
    <p style="color: green;">{{ message }}</p>
    <p>Extracted {{ count }} frames</p>
    <p><a href="/questions/{{ video_id }}">Generate Questions</a></p>
    {% else %}
    <p style="color: red;">{{ message }}</p>
    {% endif %} {% endif %}

    <p><a href="/">Back to Home</a></p>
  </body>
</html>
```

### templates/questions.html

```html
<!DOCTYPE html>
<html>
  <head>
    <title>Question Generation</title>
  </head>
  <body>
    <h1>Generate Questions - {{ video_id }}</h1>

    {% if duration_seconds %}
    <p>Video Duration: {{ duration_seconds }} seconds</p>
    {% endif %}

    <form method="post" id="questionForm">
      <div>
        <label for="start_seconds">Start Time (seconds):</label>
        <input
          type="number"
          id="start_seconds"
          name="start_seconds"
          value="{{ start_seconds or 0 }}"
          min="0"
        />
      </div>

      <div>
        <label for="interval_seconds">Interval Length (seconds):</label>
        <input
          type="number"
          id="interval_seconds"
          name="interval_seconds"
          value="{{ interval_seconds or 60 }}"
          min="1"
          required
        />
      </div>

      <div>
        <label for="full_duration">
          <input
            type="checkbox"
            id="full_duration"
            name="full_duration"
            {%
            if
            full_duration
            %}checked{%
            endif
            %}
          />
          Generate for entire video duration
        </label>
      </div>

      <div>
        <label for="api_key"
          >OpenAI API Key (optional if set as environment variable):</label
        >
        <input
          type="password"
          id="api_key"
          name="api_key"
          style="width: 400px;"
        />
      </div>

      <button type="submit">Generate Questions</button>
      <button type="button" onclick="startWebSocket()">Stream Results</button>
    </form>

    <div id="progress" style="margin-top: 20px;"></div>

    {% if error %}
    <div style="color: red; margin-top: 20px;">
      <h3>Error:</h3>
      <p>{{ error }}</p>
    </div>
    {% endif %} {% if result %}
    <div style="margin-top: 20px;">
      <h3>Generated Questions:</h3>
      <pre>{{ result }}</pre>
    </div>
    {% endif %}

    <p><a href="/">Back to Home</a></p>

    <script>
      function startWebSocket() {
        const form = document.getElementById("questionForm");
        const formData = new FormData(form);
        const progressDiv = document.getElementById("progress");

        const ws = new WebSocket(
          `ws://localhost:8000/ws/questions/{{ video_id }}`,
        );

        ws.onopen = function () {
          progressDiv.innerHTML = "<p>Connected to server...</p>";
          ws.send(
            JSON.stringify({
              start_seconds: parseInt(formData.get("start_seconds") || "0"),
              interval_seconds: parseInt(formData.get("interval_seconds")),
              full_duration: formData.get("full_duration") === "on",
              api_key: formData.get("api_key") || null,
            }),
          );
        };

        ws.onmessage = function (event) {
          const data = JSON.parse(event.data);
          if (data.type === "status") {
            progressDiv.innerHTML += `<p>${data.message}</p>`;
          } else if (data.type === "segment_result") {
            progressDiv.innerHTML += `<div><strong>Segment ${data.start}-${data.end}s:</strong><pre>${JSON.stringify(data.result, null, 2)}</pre></div>`;
          } else if (data.type === "done") {
            progressDiv.innerHTML +=
              "<p><strong>Generation complete!</strong></p>";
          } else if (data.type === "error") {
            progressDiv.innerHTML += `<p style="color: red;">Error: ${data.message}</p>`;
          }
        };

        ws.onclose = function () {
          progressDiv.innerHTML += "<p>Connection closed.</p>";
        };
      }
    </script>
  </body>
</html>
```

## Usage

### 1. Start the Application

```bash
# Make sure your virtual environment is activated
uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

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
3. Optionally enter your OpenAI API key if not set as environment variable
4. Choose between single interval or full duration processing
5. Use "Stream Results" for real-time progress updates
>>>>>>> 612e9a2 (updated readme)

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

<<<<<<< HEAD
- 403 Forbidden on download:
  - Try again (the downloader cycles player clients automatically).
  - Add a cookies file via `YTDLP_COOKIEFILE`.
  - Install Node.js LTS to improve extraction reliability.
- Low quality:
  - The first attempt prefers 720p H.264 MP4. If you need higher, adjust the
    format selector in `main.py` inside `download_youtube()`.
- FFmpeg not found:
  - Install FFmpeg and ensure it is on your PATH.
=======
### Common Issues

1. **"No module named 'cv2'"**

   ```bash
   pip install opencv-python
   ```

2. **"FFmpeg not found"**
   - yt-dlp usually handles this automatically
   - On Windows: Download FFmpeg and add to PATH
   - On macOS: `brew install ffmpeg`
   - On Ubuntu: `sudo apt install ffmpeg`

3. **OpenAI API errors**
   - Verify your API key is correct
   - Check your OpenAI account has sufficient credits
   - Ensure you have access to GPT-4 Vision API

4. **WebSocket connection issues**
   - Check firewall settings
   - Ensure the server is running on the correct port
   - Try using HTTP endpoints instead

### Performance Tips

- For long videos, use smaller intervals (30-60 seconds) to avoid API timeouts
- The application resizes images to 512x512 for efficiency
- Frame extraction can take several minutes for long videos
>>>>>>> 612e9a2 (updated readme)

## License

This project is for educational purposes. Please respect YouTube's Terms of Service and copyright laws when downloading content.

## Support

For issues related to:

- **yt-dlp**: Check [yt-dlp documentation](https://github.com/yt-dlp/yt-dlp)
- **OpenAI API**: Check [OpenAI documentation](https://platform.openai.com/docs)
- **FastAPI**: Check [FastAPI documentation](https://fastapi.tiangolo.com)

## Collaborators

- Ayush Gupta
- Riju Pant
