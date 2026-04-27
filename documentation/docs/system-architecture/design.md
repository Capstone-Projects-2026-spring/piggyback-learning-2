---
sidebar_position: 1
---


# Design

## Components
### Database Schema
```
CREATE TABLE parents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE kids (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE videos (
    id VARCHAR(255) PRIMARY KEY, -- YouTube ID
    title VARCHAR(255),
    thumbnail_url VARCHAR(2048),
    duration_seconds INTEGER,
    local_video_path VARCHAR(2048),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id VARCHAR(255) NOT NULL,
    start_seconds INTEGER NOT NULL,
    end_seconds INTEGER NOT NULL,
    best_question TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE frames (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id VARCHAR(255) NOT NULL,
    frame_number INTEGER NOT NULL,
    timestamp_seconds INTEGER NOT NULL UNIQUE,
    timestamp_formatted VARCHAR(50) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(2048) NOT NULL,
    subtitle_text TEXT,
    is_keyframe BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE questions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    segment_id INTEGER NOT NULL,
    qtype VARCHAR(50) NOT NULL,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    rank INTEGER,
    followup_enabled BOOLEAN DEFAULT 0,
    followup_correct_question TEXT,
    followup_correct_answer TEXT,
    followup_wrong_question TEXT,
    followup_wrong_answer TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (segment_id) REFERENCES segments(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE video_assignments (
    kid_id INTEGER NOT NULL,
    video_id VARCHAR(255) NOT NULL,
    answers JSON, -- Stores array of child responses
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (kid_id, video_id),
    FOREIGN KEY (kid_id) REFERENCES kids(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE video_tags (
    video_id VARCHAR(255) NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (video_id, tag_id),
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE kid_tags (
    kid_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (kid_id, tag_id),
    FOREIGN KEY (kid_id) REFERENCES kids(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE ON UPDATE CASCADE
);
```

**Class Diagram**
```mermaid
classDiagram
    direction LR
    class User {
        <<Entity>>
        +i32 id
        +String username
        +String password_hash
        +DateTime created_at
    }

    class Parent {
        <<Entity>>
        +String name
        +get_kids(parent_id)
    }

    class Kid {
        <<Entity>>
        +i32 parent_id
        +String name
        +get_recommendations()
        +get_kid_tags()
    }

    class Video {
        <<Entity>>
        +String id (Youtube ID)
        +String title
        +String thumbnail_url
        +i32 duration_seconds
        +String local_video_path
    }

    class Segment {
        <<Entity>>
        +i32 id
        +String video_id
        +i32 start_seconds
        +i32 end_seconds
        +String best_question
    }

    class Question {
        <<Entity>>
        +i32 id
        +i32 segment_id
        +String qtype
        +String question
        +String answer
        +i32 rank
    }

    class Frame {
        <<Entity>>
        +String video_id
        +i32 frame_number
        +i32 timestamp_seconds
        +String timestamp_formatted
        +String file_path
    }

    class VideoAssignment {
        <<Entity>>
        +i32 kid_id
        +String video_id
        +JsonValue answers (List of Answer)
    }

    class Answer {
        <<Struct>>
        +String transcript
        +bool is_correct
        +f32 similarity_score
        +String mood
        +f32 energy
        +i32 segment_id
    }

    class Tag {
        <<Entity>>
        +i32 id
        +String name
    }

    class VideoController {
        +download_and_store(video_id)
        +extract_frames(video_id)
        +add_video_tags(video_id, tags)
    }

    class AuthController {
        +signup(SignupData)
        +login(LoginData)
        +generate_jwt(id)
    }

    class AnswerController {
        +analyze_answer(audio, expected)
        +get_answers(kid_id, video_id)
    }

    class WS_Manager {
        +AppState users_map
        +handle_socket(stream, username)
        +broadcast(WsMessage)
    }

    User <|-- Parent
    User <|-- Kid
    Parent "1" -- "✱" Kid : manages
    
    Video "1" -- "✱" Segment : contains
    Segment "1" -- "✱" Question : contains
    Video "1" -- "✱" Frame : has extracted
    
    Kid "1" -- "✱" VideoAssignment
    VideoAssignment "1" -- "✱" Answer : stores as json
    Kid "✱" -- "✱" Tag : kid tags
    Video "✱" -- "✱" Tag : video tags

    VideoController ..> Video : creates
    AnswerController ..> VideoAssignment : updates
    WS_Manager ..> Kid : streams to
    AuthController ..> User : authenticates
```

---

**Backend**
#### Built with Loco.rs, which integrates:
- Axum – Used for Web server and routing
- SeaORM – Uses Database ORM for SQL queries
- Database: Uses SQLite for lightweight, storage.
- WebSocket support is provided by Axum for real-time communication. 

#### Authentication and Security
- Role Access Control: Implements a system to login using Parent and Kid accounts which have separate roles.
- Linking Parent and Child: Kid accounts are linked to Parent IDs via foreign keys in the database. This allows parents to monitor and assign videos to their kids.
- JWT Security: Uses JSON Web Tokens (JWT) for management pf secure sessions, with secrets managed via environment variables.

#### Video Processing
- YouTube Downloading: Uses yt-dlp to fetch videos, metadata, and subtitles from a YouTube URL.
- Frame Extraction: Uses FFmpeg to extract frames from downloaded videos for AI to used during question generation.
- Processing is done with API endpoints and progress of processing is streamed to the frontend over WebSockets.

### AI Integration
- Question Generation: Processes video metadata, transcripts/subtitles, and extracted frames to generate questions using AI.
- Answer Validation: Grades user responses (both text and transcribed audio) against expected answers. 
- OpenAI is integrated for question generation

### Speech Processing
- Speech Recognition: Uses Vosk for transcribing children's audio responses without an internet connection, and to comply with COPPA and GDPR-K laws.
- Model files are stored locally.
- Text-to-Speech (TTS): Has endpoints for generating spoken prompts and feedback for the mascot to handle the quiz.
- Mood Detection: After the child's audio response is transcribed to text. It analyzes physical properties of the transcribed audio in order to determine the child's mood. (Bored , Neutral, or Excited.) This is to comply with COPPA and GDPR-K laws.

### Static / Media Serving
Generated assets (the videos, extracted frames, question JSON files) are served statically, and can be access by URL for the frontend to use.

**Frontend**
### Page rendering
This Next.js application uses the App Router. Pages are React components inside app/, and the routing follows Next.js file-system conventions.
Main entry pages (find them in /frontend/app): 
    - kids/[id]
    - login
    - signup
    - videos

### Frontend Logic:
- The UI is built with Next.js (React) and TypeScript.
- Pages are composed from reusable components located in components/.
- Styling is handled via Tailwind CSS and CSS files.
- Shared logic (auth, WebSocket connections) is managed through React Context (context/) and custom hooks (hooks/).

### Flow of Data: 
- The browser loads the initial HTML from Next.js.
- Client‑side uses fetch() (or libraries like axios) to call the backend REST API.
- Real‑time updates are delivered with WebSockets using a custom hook/context.

### Parent page: 
- Processes videos (download, frame extraction, and AI question generation) via API calls.
- Opens a WebSocket connection to stream progress updates in real time (no polling).
- Fetches available videos and generated questions from the backend.
- Review, edit, and finalize questions via API calls. 
- Access child statistics such as quiz performance, mood, and distraction metrics.

### Kids page
- Loads the video catalog and quiz data.
- Calls TTS (text‑to‑speech) endpoints for spoken prompts and feedback.
- Record audio responses and transcribes them with backend APIs.

### Attention and Gaze Tracking
 - Uses Google MediaPipe (FaceLandmarker) for local eye tracking.
- Privacy Law Compliance: Processing is done entirely on the client side. Raw camera data is never sent to the server.
- Attention Logic: If the application detects the child has looked away for a period of time,an automatic video pause os triggered and the app sends a distraction notification to the parent through the backend.

### Static/Media Serving:
- Static assets are reachable from the public/ directory (Next.js convention).
- videos, extracted frames, question JSON can be reached by the backend and accessed via API routes.
- Next.js serves static assets while Rust serves the generated content

---

**Important Distinctions:**
- Documentation/ is a different frontend project: Docusaurus + React.
- That react site is for docs only and has it own build/runtime flow.
- The backend (Loco.rs / Axum) gives both REST APIs and WebSocket endpoints for the frontend. 

---
