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
