---
sidebar_position: 7
---

# ERD

```mermaid
erDiagram
    PARENTS ||--o{ KIDS : "manages"
    TAGS ||--o{ KID_TAGS : "describes"
    TAGS ||--o{ VIDEO_TAGS : "applied to"

    KIDS ||--o{ KID_TAGS : "interested in"
    VIDEOS ||--o{ VIDEO_TAGS : "categorized by"
    
    KIDS ||--o{ VIDEO_ASSIGNMENTS : "is assigned"
    VIDEOS ||--o{ VIDEO_ASSIGNMENTS : "assigned to kids"
    VIDEOS ||--o{ SEGMENTS : "contains"
    VIDEOS ||--o{ FRAMES : "has"

    SEGMENTS ||--o{ QUESTIONS : "has"

    PARENTS {
        int id PK
        string username
        string password_hash
        string name
        datetime created_at
        datetime updated_at
    }

    KIDS {
        int id PK
        int parent_id FK
        string username
        string password_hash
        string name
        datetime created_at
        datetime updated_at
    }

    VIDEOS {
        string id PK "YouTube ID"
        string title
        string thumbnail_url
        int duration_seconds
        string local_video_path
        datetime created_at
        datetime updated_at
    }

    SEGMENTS {
        int id PK
        string video_id FK
        int start_seconds
        int end_seconds
        string best_question
        datetime created_at
        datetime updated_at
    }

    FRAMES {
        int id PK
        string video_id FK
        int frame_number
        int timestamp_seconds UK
        string timestamp_formatted
        string filename
        string file_path
        string subtitle_text
        bool is_keyframe
        datetime created_at
        datetime updated_at
    }

    QUESTIONS {
        int id PK
        int segment_id FK
        string qtype
        string question
        string answer
        int rank
        bool followup_enabled
        string followup_correct_question
        string followup_correct_answer
        string followup_wrong_question
        string followup_wrong_answer
        datetime created_at
        datetime updated_at
    }

    VIDEO_ASSIGNMENTS {
        int kid_id PK, FK
        string video_id PK, FK
        json answers "List of Answer objects"
        datetime created_at
        datetime updated_at
    }

    TAGS {
        int id PK
        string name UK
        datetime created_at
        datetime updated_at
    }

    VIDEO_TAGS {
        string video_id PK, FK
        int tag_id PK, FK
        datetime created_at
        datetime updated_at
    }

    KID_TAGS {
        int kid_id PK, FK
        int tag_id PK, FK
        datetime created_at
        datetime updated_at
    }
```
## ERD Diagram Explanation
1. Parent to Kids : A parent account may add multiple kids attached to their account and manage them but start with no kids. A Kid account may only be attached to one parent account. This allows parent accounts to ,manage kid accounts.
2. Kids to Kid_Tags : Parents can attach multiple tags to the Kid account, relating to what the kid is interested in. 
3. Tags to Kid_Tags : Kid Tags link a Kid account to the main tag which describes the tag definition. This is used in the reccomendation algorithm. The algorithm connects what the Kid is interested in, to the video with what they are interested in.
4. Tags to Video_Tags : Video tags link the main tag to the Videos entity. A video may have multiple tags on it, but each tag on the video is linked to a main tag. Main tag describes what the tag is. This is also used in the reccomendation algorithm.
5. Videos to Video_Tags : Each Video can have multiple tags attached to it, relating to what the topic of the video is about. Parents can add new tags for future users.
6. Video to Video Assignments to Kids : Multiple videos can be assigned to Kids by the parent account. Each Kid account can have multiple videos assigned to them. Each Video Assignment must assign only one video to one kid. 
7. Videos to Frames : Each Video has multiple frames which are sent to AI to generate questions for the videos.
8. Videos to Segments to Questions : Each Video is split into many segments where each segment can have multiple questions attached to them in order to quiz the kid.
