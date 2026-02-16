---
sidebar_position: 4
---

# Development Environment

## Hardware Requirements

### Typical Hardware

- A modern laptop or desktop capable of running Python and FFmpeg

- Minimum suggested 8 GBs of RAM

- Enough disk space to store downloaded video data and extracted frames

- Internet access for installing dependencies and downloading YouTube content

### Setup Effort

**Installing Python 3.10+**

- Downloading and installing from python.org or OS package manager.

**Installing FFmpeg**

- Is required for multimedia processing.

**Installing Node.js (optional)**

- Improves yt-dlp reliability.

**Cloning Git Repo**

- Using Git commands to clone the repo.

**Creating a .env config file**

- Stores the secret keys and passwords.

## Software Requirements

### Programming Languages and Frameworks

python 3.x - Core backend language (FastAPI framework)

FastAPI - Web server framework used to build APIs

Unicorn - ASGI server for running the FastAPI app

### Dependencies

FFmpeg - System-level dependency for media processing

Node.js - Optional but recommended for better yt-dlp performance

### Python Libraries

Web & ASGI: fastapi, uvicorn, starlette

Multimedia: yt-dlp, opencv-python, pandas, numpy

Data & Utils: openai, gemini, python-dotenv, tqdm, python-multipart

Networking & Async: httpx, anyio, websockets

## Tools & IDEs

### IDE / Code Editor:

- VS code, PyCharm, Sublime Text, or any editor that supports Python

### Terminal / Shell:

- Git Bash (Windows) - Repo includes git bash for setup

## Local Setup Workflow

### Install dependenices

python -m venv venv

source venv/bin/activate

python -m pip install --upgrade pip

python -m pip install -r requirements.txt

### Run the Application

python -m uvicorn main:app --reload --host 0.0.0.0 --port 8000

### Verify system tools

ffmpeg -version

node -v
