---
sidebar_position: 4
---
A quick view of what tests are passing / failing.


# Unit tests

## videos/tests.py

### Tests for videos/urls.py 
#### 1. `test_kids_videos_endpoint`
##### PASSED
#### 2. `test_video_list_endpoint`
##### PASSED

### Tests for videos/views.py
#### 1. `test_kids_videos_success`
##### PASSED
#### 2. `test_kids_videos_duration_format`
##### PASSED
#### 3. `test_kids_videos_empty_db`
##### PASSED

# Integration tests

# Acceptance tests
Automated acceptance tests were performed using `curl` to validate system accessibility and endpoint behavior.

### Automated Tests

| Test ID | Description | Result |
|----------|-------------|--------|
| AT-AUTO-01 | Homepage loads successfully | PASS |
| AT-AUTO-02 | API documentation accessible | PASS |
| AT-AUTO-03 | OpenAPI schema generated | PASS |
| AT-AUTO-04 | Admin route requires authentication | PASS |

All automated acceptance tests returned expected HTTP status codes and responses.

---

### Manual Verification

Manual browser testing confirmed:

- Homepage renders correctly
- Swagger UI loads properly
- OpenAPI schema displays correctly
- Admin route redirects to login page

All manual verification tests passed.