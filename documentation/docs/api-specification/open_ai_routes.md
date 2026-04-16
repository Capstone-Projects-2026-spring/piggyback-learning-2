---
sidebar_position: 7
---

# Open AI routes
Purpose: to establish communication with AI to generate specific question types based on the video

## ``` GET /api/openai/{video_id} ```

### Parameters

#### Path Parameters
| Name | Type | Description |
| :--- | :--- | :--- |
| `video_id` | String | ID of the video  |

#### Query-String Parameters
| Name | Type | Description |
| :--- | :--- | :--- |
| `start` | int32 | Start of the segment in seconds |
| `end` | int32 | End of the segment in seconds |

### Response 200 OK Schema
```
{
    questions: [{
        answer: string
        qtype: string
        question: string
        rank: integer or null
    }]
    segment: {
        best_question: string or null
        end_seconds: integer
        id: integer
        start_seconds: integer
        video_id: string
    }
}
```
### Response 200 OK example, Questions generated successfully
```
{
    "questions": [
        {
            "answer": "string",
            "qtype": "string",
            "question": "string",
            "rank": 0
        }
    ],
    "segment": {
        "best_question": "string",
        "end_seconds": 0,
        "id": 0,
        "start_seconds": 0,
        "video_id": "string"
    }
}
```
## Response 500 errror message
Unknown error occurred
