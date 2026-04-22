---
sidebar_position: 1
---


# Design

## Components
### Database
### Entity Relationship Diagram:

```mermaid
erDiagram
    USERS {
        INTEGER id PK
        VARCHAR email
        VARCHAR username
        VARCHAR password_hash
        INTEGER role_id FK
        TIMESTAMP created_at
        TIMESTAMP updated_at
        TIMESTAMP last_login
    }

    USER_PROFILES {
        INTEGER id PK
        INTEGER user_id FK
        VARCHAR first_name
        VARCHAR last_name
        TIMESTAMP date_of_birth
    }

    ROLES {
        INTEGER id PK
        VARCHAR role_type
    }

    VIDEOS {
        INTEGER id PK
        VARCHAR url
        VARCHAR title
        INTEGER duration_in_seconds
    }

    WATCH_HISTORIES {
        INTEGER id PK
        INTEGER user_id FK
        INTEGER video_id FK
        TIMESTAMP watched_on
    }

    ROLES ||--o{ USERS : has
    USERS ||--|| USER_PROFILES : has
    USERS ||--o{ WATCH_HISTORIES : watches
    VIDEOS ||--o{ WATCH_HISTORIES : watched_in
```


Initial database schema:
```
CREATE TABLE roles (
    id INTEGER PRIMARY KEY,
    role_type VARCHAR(30) NOT NULL UNIQUE
);

CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(60) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES roles(id)
);

CREATE TABLE user_profiles (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    first_name VARCHAR(60),
    last_name VARCHAR(100),
    date_of_birth TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE videos (
    id INTEGER PRIMARY KEY,
    url VARCHAR(2048) NOT NULL,
    title VARCHAR(300) NOT NULL,
    duration_in_seconds INTEGER NOT NULL
);

CREATE TABLE watch_histories (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    video_id INTEGER NOT NULL,
    watched_on TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE
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
### Framework & Stack
#### Built with Loco.rs, which integrates:
- Axum – Used for Web server and routing
- SeaORM – Uses Database ORM for SQL queries
- Database: Uses SQLite for lightweight, storage.
- WebSocket support is provided by Axum for real-time communication. 

### Core Functionality
#### Video Processing
- YouTube Downloading: Uses yt-dlp to fetch videos, metadata, and subtitles from a YouTube URL.
- Frame Extraction: Uses FFmpeg to extract frames from downloaded videos for AI to used during question generation.
- Processing is done with API endpoints and progress of processing is streamed to the frontend over WebSockets.

### AI Integration
- Question Generation: Processes video metadata, transcripts/subtitles, and extracted frames to generate questions using AI.
- Answer Validation: Grades user responses (both text and transcribed audio) against expected answers. 
- OpenAI is integrated for question generation

### Speech Processing
- Speech Recognition: Uses Vosk for transcribing children's audio responses without an internet connection, and to comply with COPPA and other similar laws.
- Model files are stored locally.
- Text-to-Speech (TTS): Has endpoints for generating spoken prompts and feedback for the mascot to handle the quiz.

### Static / Media Serving
Generated assets (the videos, extracted frames, question JSON files) are served statically, and can be access by URL for the frontend to use.

**Frontend**

1. Page rendering : 
This Next.js application uses the App Router. Pages are React components inside app/, and the routing follows Next.js file-system conventions.
Main entry pages (find them in /frontend/app): 
    - kids/[id]
    - login
    - signup
    - videos

2. Frontend Technology:
- The UI is built with Next.js (React) and TypeScript.
- Pages are composed from reusable components located in components/.
- Styling is handled via Tailwind CSS and CSS files.
- Shared logic (auth, WebSocket connections) is managed through React Context (context/) and custom hooks (hooks/).

3. Data flow: 
- The browser loads the initial HTML from Next.js.
- Client‑side uses fetch() (or libraries like axios) to call the backend REST API.
- Real‑time updates are delivered with WebSockets using a custom hook/context.

4. Parent page: 
- Processes videos (download, frame extraction, and AI question generation) via API calls.
- Opens a WebSocket connection to stream progress updates in real time (no polling).
- Fetches available videos and generated questions from the backend.
- Review, edit, and finalize questions via API calls. 

5. Kids page
- Loads the video catalog and quiz data.
- Calls TTS (text‑to‑speech) endpoints for spoken prompts and feedback.
- Record audio responses and transcribes them with backend APIs.

6. Static/Media Serving:
- Static assets are reachable from the public/ directory (Next.js convention).
- videos, extracted frames, question JSON can be reached by the backend and accessed via API routes.
- Next.js serves static assets while Rust serves the generated content

---

**Important Distinctions:**
- Documentation/ is a different frontend project: Docusaurus + React.
- That react site is for docs only and has it own build/runtime flow.
- The backend (Loco.rs / Axum) gives both REST APIs and WebSocket endpoints for the frontend. 

---
