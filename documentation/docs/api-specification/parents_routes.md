---
sidebar_position: 8
---
# Parent Routes
Purpose: to get kids that belong to the parent so that parents can perform operations on the kids account such as assigning videos.

## ``` GET /api/parents/{parent_id}/kids ```

### Path Parameters
| Name | Type | Description |
| :--- | :--- | :--- |
| `parent_id` | int32 | Unique ID for the Parent |


### Response 200 OK Schema
```
[{
    created_at: date-time
    id: integer
    name: string
    updated_at: date-time
}]
```
### Response 200 OK example, Kids belonging to the parent
```
[
    {
        "created_at": "2026-01-15T10:30:00Z",
        "id": 0,
        "name": "string",
        "updated_at": "2026-01-15T10:30:00Z"
    }
]
```