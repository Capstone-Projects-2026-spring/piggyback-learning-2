---
sidebar_position: 12
---

# Websocket Connection

## GET /ws

### Query-String Parameters
| Name | Type |
| ____ | ____ |
|  username | string |

### Response 101 Schema
```
{
    action: string
    msg: string
    receiver: string
    sender: string
}
```

### Response 101 example, WebSocket connection established
```
{
    "action": "string",
    "msg": "string",
    "receiver": "string",
    "sender": "string"
}
```

### Response 500 error message
Bad request
