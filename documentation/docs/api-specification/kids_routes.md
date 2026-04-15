---
sidebar_position: 6
---

# Frame routes
Purpose: Used for getting and adding tags for the kids, assigning videos to kids, and for getting videos that were reccomended.

## GET /api/kids/{kid_id}/recommendations

### Request Path Parameters
| Name | Type |
| ____ | ____ |
| kid_id | int32 |

### Response 200 OK Schema
{
    recommendations*: [{
        duration_seconds: integer or null
        id*: string
        score*: integer
        thumbnail_url: string or null
        title: string or null
    }]
    tags*: [string]
}

### Response 200 OK example, Recommended videos based on kid's tags
{
    "recommendations": [
        {
            "duration_seconds": 0,
            "id": "string",
            "score": 0,
            "thumbnail_url": "string",
            "title": "string"
        }
    ],
    "tags": [
        "string"
    ]
}



## GET /api/kids/{kid_id}/tags

### Request Path Parameters
| Name | Type |
| ____ | ____ |
| kid_id | int32 |

### Response 200 OK Schema
[{
    id*: integer
    name*: string
}]

### Response 200 OK example, Tags for the kid
[
    {
        "id": 0,
        "name": "string"
    }
]


## POST /api/kids/{kid_id}/tags

### Request Path Parameters
| Name | Type |
| ____ | ____ |
| kid_id | int32 |

## Request Body
{
  "tags": [
    0
  ]
}

### Response 200 OK Schema
{
    success*: boolean
}


### Response 200 OK example, Tags added successfully
{
    "success": false
}

### Response 500 error message
Unknown error occurred



## GET /api/kids/{kid_id}/videos_assigned

### Request Path Parameters
| Name | Type |
| ____ | ____ |
| kid_id | int32 |

## Request Body
{
  "tags": [
    0
  ]
}

### Response 200 OK Schema
[{
    created_at*: date-time
    id*: integer
    name*: string
    updated_at*: date-time
}]

### Response 200 OK example, Videos assigned to the kid
[
    {
        "created_at": "2026-01-15T10:30:00Z",
        "id": 0,
        "name": "string",
        "updated_at": "2026-01-15T10:30:00Z"
    }
]

## POST /api/kids/{kid_id}/videos_assigned

### Request Path Parameters
| Name | Type |
| ____ | ____ |
| kid_id | int32 |

## Request Body
{
  "video_id": "string"
}

### Response 200 OK Schema
{
success*: boolean
}

### Response 200 OK example, Video assigned successfully
{
    "success": false
}

### Response 500 error message
Unknown error occurred