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

## user/views.py

### Tests for user/views.py
#### 1. `test_verify_password_success_admin`
##### PASSED
#### 2. `test_verify_password_success_expert`
##### PASSED
#### test_verify_password_failure_invalid_password
##### PASSED

## ai/views.py
### Tests for ai/views.py
#### 3. `test_verify_password_failure_invalid_password`
##### PASSED
#### 4. `test_check_answer_correct_exact_match`
##### PASSED
#### 5. `test_check_answer_numeric_match`
##### PASSED
#### 6. `test_check_answer_numeric_mismatch`
##### PASSED
#### 7. `test_check_answer_missing_numeric_answer`
##### PASSED
#### 8. `test_check_answer_correct_non_numeric`
##### PASSED
#### 9. `test_check_answer_list_partial_items_matched`
##### PASSED
#### 10. `test_check_answer_missing_input`
##### PASSED
#### 11. `test_check_answer_high_similarity`
##### PASSED
#### 12. `test_check_answer_low_similarity`
##### PASSED
#### 13. `test_check_answer_borderline_similarity`
##### PASSED

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