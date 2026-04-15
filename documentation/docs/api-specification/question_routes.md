---
sidebar_position: 9
---

# Question Routes
Purpose: to get all the questions associated with a video, that are generated and approved by a human.

## ``` GET /api/questions/{video_id} ```

### Path Parameters
| Name | Type |
| ____ | ____ |
| video_id | int32 |

### Response 200 OK Schema
```
{
    segments: [{
        best_question: string or null
        end_seconds: integer
        id: integer
        questions: [{
            answer: string
            qtype: string
            question: string
            rank: integer or null
        }]
        start_seconds: integer
    }]
    video_id: string
}
```
### Response 200 OK example, Questions grouped by segment for the video
```
{
    "segments": [
        {
        "best_question": "string",
        "end_seconds": 0,
        "id": 0,
        "questions": [
            {
                "answer": "string",
                "qtype": "string",
                "question": "string",
                "rank": 0
            }
        ],
        "start_seconds": 0
        }
    ],
    "video_id": "string"
}
```