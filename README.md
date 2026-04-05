# Piggyback Learning

Piggyback Learning is a full-stack educational platform designed to help children (ages 6–8) improve comprehension skills through interactive video-based quizzes.

The system processes YouTube videos, generates questions using AI, and provides a kid-friendly interface for answering questions with support for speech, playback tracking, and real-time interactions.

---

## Architecture

The project is split into two main parts:

- **Frontend** — Built with Next.js (React) for an interactive user experience  
- **Backend** — Built with Loco.rs (Rust, Axum, SeaORM) for high-performance APIs, video processing, and AI integration  

### Repositories

- Frontend setup and development: [Frontend Setup](frontend/README.md)
- Backend setup and development: [Backend Setup](backend/README.md)

> Follow the instructions in each README to run the full application locally.

---

## Features

### Core Functionality

- YouTube video ingestion and processing (`yt-dlp`, FFmpeg)
- AI-powered question generation
- Answer validation and grading
- Speech recording and transcription
- Real-time interactions via WebSockets
- Kid-friendly quiz interface

### Learning Experience

- Interactive video playback with timed questions
- Audio-based responses and feedback
- Progress tracking and engagement monitoring

### System Capabilities

- REST API backend (Axum)
- WebSocket support for live updates
- SQLite database via SeaORM
- Offline speech recognition using Vosk

---

## How It Works

1. **Backend**
 - Downloads and processes videos
 - Generates questions using AI
 - Stores structured quiz data
 - Provides REST + WebSocket APIs

2. **Frontend**
 - Displays videos and quizzes
 - Captures user responses (text/audio)
 - Communicates with backend APIs
 - Streams real-time updates

---

## Running the Project

To run the full application locally:

### 1. Start the Backend

Follow: [Backend Setup](backend/README.md)

The backend runs at: http://localhost:5150

API docs: http://localhost:5150/docs

---

### 2. Start the Frontend

Follow: [Frontend Setup](frontend/README.md)

The frontend runs at: http://localhost:3000

---

### 3. Connect Frontend to Backend

Ensure your frontend environment variables are set:

```env
NEXT_PUBLIC_API_BASE_URL=http://localhost:5150
NEXT_PUBLIC_WS_BASE_URL=ws://localhost:5150
```

----

### Project Structure

```
piggyback-learning-2/
├── frontend/          # Next.js application
├── backend/           # Loco.rs (Rust) backend
└── README.md          # Root project overview (this file)
```

---

### Technologies Used

Frontend
  - Next.js (React)
  - WebSockets
  - Modern React hooks + context

Backend
  - Loco.rs (Axum)
  - SeaORM (SQLite)
  - Vosk (offline speech recognition)
  - FFmpeg + yt-dlp

---

## Collaborators

- Ayush Gupta
- Riju Pant
- William Yang
- Adam Marx
- Shiven Patel
