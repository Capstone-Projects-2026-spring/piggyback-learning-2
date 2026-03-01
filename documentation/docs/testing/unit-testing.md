---
sidebar_position: 1
---
# Unit tests
You can run these tests yourself by running 
```python manage.py test``` 
in the terminal (in the directory of the project)

## videos/tests.py

### Overview

This test suite validates:

- URL routing and endpoint availability
- Correct JSON response structure
- Data formatting logic (duration formatting)
- Behavior when database is empty

The tests use Django’s `TestCase` and Django REST Framework status codes.

---

### Test Data Setup

#### Method: `setUp()`

##### Purpose
Creates two `Video` objects in the test database before each test runs.

##### Expected Behavior
- Test database contains exactly two video records.
- Each test starts with a clean database state.

---

### Tests for videos/urls.py 

#### 1. `test_kids_videos_endpoint`

##### Purpose
Verifies that the `kids-videos` endpoint is reachable.

##### Expected Behavior
- Endpoint exists
- Endpoint responds successfully

---

#### 2. `test_video_list_endpoint`

##### Purpose
Verifies that the `video-list` endpoint is reachable.

##### Expected Behavior
- Endpoint exists
- Endpoint responds successfully

---

### Tests for videos/views.py

#### 1. `test_kids_videos_success`

##### Purpose
Ensures the `kids-videos` endpoint returns correct JSON structure and data.

##### Expected behaviour
`kids-videos` endpoint returns 2 videos and status code 200


#### 2. `test_kids_videos_duration_format`

##### Purpose
Verifies the property `duration` can be read as `minutes:seconds` or `2:05` for example

##### Expected behaviour
The `duration` of the first video recieved is equal to `2:05`


#### 3. `test_kids_videos_empty_db`

##### Purpose
Verifies that `kids-videos` still returns 200 when there are no videos to return

##### Expected behaviour
There are 0 returned videos and the status code is 200
