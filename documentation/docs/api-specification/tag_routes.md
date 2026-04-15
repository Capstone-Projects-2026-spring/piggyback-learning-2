---
sidebar_position: 10
---

# Tag Routes
Purpose: To retrieve all tags and create new tags

## GET /api/tags

### Parameters
None

### Response 200 OK Schema
[{
    created_at*: date-time
    id*: integer
    name*: string
    updated_at*: date-time
}]

### Response 200 OK example, All tags
[
    {
        "created_at": "2026-01-15T10:30:00Z",
        "id": 0,
        "name": "string",
        "updated_at": "2026-01-15T10:30:00Z"
    }
]

## POST /api/tags

### Parameters
#### Request Body Schema
{
    name*: string
}

### Response 200 OK Schema
{
    created_at*: date-time
    id*: integer
    name*: string
    updated_at*: date-time
}

### Response 200 OK example, Tag created or returned if already exists
{
    "created_at": "2026-01-15T10:30:00Z",
    "id": 0,
    "name": "string",
    "updated_at": "2026-01-15T10:30:00Z"
}

