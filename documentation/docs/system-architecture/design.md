---
sidebar_position: 1
---


# Design

## Components
### Database
### Entity Relationship Diagram:
```
mermaid erDiagram


    USERS {
        INTEGER id PK
        VARCHAR(255) email
        VARCHAR(60) username
        VARCHAR(255) password_hash
        INTEGER role_id FK
        TIMESTAMP created_at
        TIMESTAMP updated_at
        TIMESTAMP last_login
    }

    USER_PROFILES {
        INTEGER id PK
        INTEGER user_id FK
        VARCHAR(60) first_name
        VARCHAR(100) last_name
        TIMESTAMP date_of_birth
    }

    ROLES {
        INTEGER id PK
        VARCHAR(30) role_type
    }

    VIDEOS {
        INTEGER id PK
        VARCHAR(2048) url
        VARCHAR(300) title
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
## Piggyback Learning API

**Endpoints**

```
- POST /api/verify-password
- POST /api/download
- POST /api/frames/{video_id}
- GET /api/admin/videos
- POST /api/submit-questions
- WS /ws/questions/{video_id}
- GET /api/kids_videos
- GET /api/final-questions/{video_id}
- GET /api/videos-list
- GET /api/expert-questions/{video_id}
```

**Class Diagram**
I think we should forgo a class diagram for the api for now:
- It’s written in python therefore has no classes
- It’s already developed / difficult to decipher

---

**Backend**
Youtube Downloading: Uses yt_dlp to download video, metadata, and subtitles. 

---

**Frontend**

1. Page rendering : 
FastAPI serves Jinja template from templates/ via JinjaTemplates (somewhere in main)
Main entry pages (search for them in main.py) : 
	- home.html (around line 1021)
	- children.html(around line 1020-1024?)
	- expert_preview.html(around line 1049) 
	- admin.html(around line 233) 

2. Frontend tech:
- The app UI is rendered as separate HTML files in "templates/"
- Each page includes its own "style" and "script" blocks, so behavior and styling live in that file instead of shared JS/CSS bundles.
- There is no template inheritance layer, so repeated UI patterns are manually duplicated across pages. (THIS IS BAD)
- Result: simple to reason about per page but harder to keep global consistency when changing shared style/interactions.

Data flow: 
- The browser loads and HTML landing page first. Then uses fetch() to call FastAPI endpoints and update UI dynamically.

Admin page: 
- Start processing flow(download/frame extraction /question generation) with API calls in admin.html
- Opens a Websocket in admin.html , to stream progress update in real time(instead of polling).

Expert page: 
- Pull available videos/questions and submits review/finalization changes through API calls in expert_preview.html. 

Kids page:
- Loads the kid-facing catalog and quiz data from backend endpoints in children.html
- Calls TTS endpoint for spoken prompts/feedback in children.html.

Static/Media Serving:
- main(main.py) mounts /downloads, so generated assets(videos, extracted frames, question JSON files) are directly reachable by URL.
- main also mounts /static for normal static resource (JSON/assets used by pages).

Practical meaning to all this: frontend doesn’t need a separate file server; FastAPI serves both APIs and files.
1. Serves API endpoints 
2. Serves files like videos/static assets via /downloads and /static

---

**Important Distinctions:**
- Documentation/ is a different frontend project: Docusaurus + React.
- That react site is for docs only and has it own build/runtime flow.
- Main product UI is not React; it’s server-rendered templates + vanilla JS in the FastAPI app. 

---

## Use Case Sequence Diagrams

**Use Case 1 | Answering a Quiz Question**

*As a user, I want to be able to answer quiz questions with voice recognition.*

1. A quiz for the video starts and asks the user a question.
2. The user answers vocally, after seeing a visual indication that voice input is being accepted (“you can speak now!” or something like that).
3. The user's input is mapped to an answer for the quiz.
4. If incorrect, a fallback option is triggered. Potentially a multiple-choice quiz.
5. If correct, the video continues playing.

**Use Case 2 | Assign videos**

*As a parent, I want to search for other users(kid accounts) so I can assign videos and quizzes to them.*

1. The parent opens up a search bar and type in a username/email.
2. The system displays matching accounts.
3. The parent selects the user and types the video name into another search bar for videos.
4. The system displays matching videos.
5. The parent then clicks on the video and a menu pops up.
6. From the menu, the parent chooses fallback options, toggles no distraction mode, and assigns it to the user.
7. User receives the video and gets notified.

**Use Case 3 | View Dashboard**

*As a user/parent, I want a dashboard so I can view the history and quiz performance.*

1. The user clicks on the button for dashboard.
2. The application makes a request to the database for information about the user’s history and quiz performances.
3. The application receives the data and places it into a dashboard.
4. The dashboard is then shown to the user.
5. The user is able to click on individual videos from the dashboard to see detailed stats for past videos watched.
