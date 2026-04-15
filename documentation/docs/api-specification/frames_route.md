---
sidebar_position: 5
---

# Frame routes
Purpose: used for extracting frames for the video, which will be analyzed by the AI later to generate questions.

## GET /api/frames/extract/{video_id}

### Request Path Parameters
| Name | Type |
| ____ | ____ |
| video_id | String |

### Response 200 OK Schema
{
    msg*: string
    success*: boolean
}

### Response 200 OK example, Frames extracted or already exist
{
    "msg": "string",
    "success": false
}

### Response 400 Error message example
FFMPEG failed