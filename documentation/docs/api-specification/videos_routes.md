---
sidebar_position: 11
---

# Video Routes
Purpose: To download videos, get tags for a video, and to attach tags to a video

## GET /api/videos/download/{video_id}

### Path Parameters
| Name | Type |
| ____ | ____ |
| video_id | string |

### Response 200 OK Schema
{
    success*: boolean
}

### Response 200 OK example, Video downloaded or already exists
{
    "success": false
}

### Response 500 error message
Download failed


## GET /api/videos/{video_id}/tags

### Path Parameters
| Name | Type |
| ____ | ____ |
| video_id | string |

### Response 200 OK Schema
[{
    id*: integer
    name*: string
}]

### Response 200 OK example, Tags for the video
[
    {
        "id": 0,
        "name": "string"
    }
]

## POST /api/videos/{video_id}/tags

### Parameters
#### Request Body Schema
{
    tags*: [integer]
}

### Response 200 OK Schema
{
    success*: boolean
}

### Response 200 OK example, Tags added to video
{
    "success": false
}

### Response 500 error message
Unknown error occurred
