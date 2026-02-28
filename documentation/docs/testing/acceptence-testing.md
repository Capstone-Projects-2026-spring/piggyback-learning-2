---
sidebar_position: 3\
---

# Acceptance Test

### AT-AUTO-01: Homepage Loads Successfully

**Command Executed:**\
```bash\
curl -i http://127.0.0.1:8000/

**Observed Result:**

-   HTTP/1.1 200 OK

-   HTML page rendered successfully

-   Page title displayed: "Educational Video Platform"

**Status:** PASS

* * * * *

### AT-AUTO-02: API Documentation Accessible

**Command Executed:**

curl -i http://127.0.0.1:8000/api/docs/

**Observed Result:**

-   HTTP/1.1 200 OK

-   Swagger UI documentation loaded successfully

**Status:** PASS

* * * * *

### AT-AUTO-03: OpenAPI Schema Generated Successfully

**Command Executed:**

curl -i http://127.0.0.1:8000/api/schema/

**Observed Result:**

-   HTTP/1.1 200 OK

-   OpenAPI 3.0.3 schema returned

-   API endpoints listed correctly

**Status:** PASS

* * * * *

### AT-AUTO-04: Protected Admin Route Requires Authentication

**Command Executed:**

curl -i http://127.0.0.1:8000/django-admin/

**Observed Result:**

-   HTTP/1.1 302 Found

-   Redirected to `/django-admin/login/?next=/django-admin/`

-   Authentication enforced properly

**Status:** PASS

## Manual Acceptance Confirmation

The following tests were manually verified in a web browser.

| Test ID | Description | Performed | Result | Observed Outcome |
|---------|------------|-----------|--------|-----------------|
| AT-01 | Homepage loads in browser | Yes | PASS | Page rendered successfully |
| AT-02 | API documentation accessible | Yes | PASS | Swagger UI displayed |
| AT-03 | OpenAPI schema accessible | Yes | PASS | Schema displayed |
| AT-04 | Admin route redirects to login | Yes | PASS | Login page shown |