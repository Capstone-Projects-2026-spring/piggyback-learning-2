---
sidebar_position: 4
---

# Answers Routes
Purpose: used for getting the answers to the quiz and checking the user's answers.

## POST /api/answers/analyze


## Request Body
Content-Type: multipart/form-data

| Name | Type | Description |
| :--- | :--- | :--- |
| `audio` | File[] | Array of audio files to be analyzed. |
| `expected_answer` | String | The correct text expected. |
| `kid_id` | Integer | Unique ID for the child. |
| `segment_id` | Integer | ID of the quiz segment. |
| `video_id` | String | ID of the video associated with the quiz. |

### Response 200 OK Schema
```
{
    energy: number
    is_correct: boolean
    mood: string
    similarity_score: number
    transcript: string
}
```
### Response 200 OK example, Answer analyzed successfully
```
{
    "energy": 0,
    "is_correct": false,
    "mood": "string",
    "similarity_score": 0,
    "transcript": "string"
}
```
### Response 400 Error message example
Missing or invalid fields



### ``` GET /api/answers/{kid_id}/{video_id} ```


### Parameters

Request path is multipart/form-data which consists of the portions below:

| Name | Type | Description |
| :--- | :--- | :--- |
| `kid_id `| int32 | Unique ID for the child |
| `video_id` | String | ID of the video associated with the quiz. |

### Response 200 OK Schema
```
{
    ONE OF
    1 null
    2 {
        created_at: date-time
        id: integer
        name: string
        updated_at: date-time
    }
}
```
### Response 200 OK example, Answer found or null
```
{
    "created_at": "2026-01-15T10:30:00Z",
    "id": 0,
    "name": "string",
    "updated_at": "2026-01-15T10:30:00Z"
}
```
