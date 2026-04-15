---
sidebar_position: 3
---

# Authentication routes
Purpose: Used for login and registration purposes. Authenticates usernames and passwords.

## POST /api/auth/login

### Request Body Schema
```
{
    password: string
    role: string
    username: string
}
```
### Response 200 Schema
```
{
    account: {
        ONE OF
        1 {
            created_at: date-time
            id: integer
            name: string
            updated_at: date-time
        }
        2 {
            created_at: date-time
            id: integer
            name: string
            updated_at: date-time
        }
    }
    parent_username: string or null
    role: string
    success: boolean
    token: string
}
```
### Response 200 OK example, Logged in successfully
```
{
    "account": {
        "created_at": "2026-01-15T10:30:00Z",
        "id": 0,
        "name": "string",
        "updated_at": "2026-01-15T10:30:00Z"
    },
    "parent_username": "string",
    "role": "string",
    "success": false,
    "token": "string"
}
```
### Response 400 Error message example
Invalid credentials or role

## POST /api/auth/signup

### Request body schema
```
{
    name: string
    parent_id: integer or null
    password: string
    role: string
    username: string
}
```
### Response 200 Schema
```
{
    success: boolean
}
```
### Response 200 OK example, Signed up successfully
```
{
    "success": false
}
```
### Response 400 Error message example
Invalid role or missing parent_id for kid
