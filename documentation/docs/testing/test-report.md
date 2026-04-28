---
sidebar_position: 4
---

A quick view of what tests are passing / failing.

# Unit Tests

## models/

| Model | Test | Result |
|------|------|--------|
| kid_tags | test_model | PASSED |
| kids | kid_parent_relationship | PASSED |
| parents | test_model | PASSED |
| questions | test_model | PASSED |
| frames | test_model | PASSED |
| generated_questions | test_model | PASSED |
| segments | test_model | PASSED |
| tags | test_model | PASSED |
| video_assignments | test_model | PASSED |
| videos | test_model | PASSED |
| video_tags | test_model | PASSED |

---

# Integration Tests

## tests/requests/auth.rs

| Category | Test | Result |
|----------|------|--------|
| Authentication Routes | can_signup_parent | PASSED |
| Authentication Routes | can_signup_kid | PASSED |
| Authentication Routes | duplicate_parent_username_rejected | PASSED |
| Authentication Routes | duplicate_kid_username_rejected | PASSED |
| Authentication Routes | kid_signup_without_parent_id_rejected | PASSED |
| Authentication Routes | invalid_role_rejected | PASSED |
| Authentication Routes | can_login_parent | PASSED |
| Authentication Routes | can_login_kid | PASSED |
| Authentication Routes | login_wrong_password_rejected | PASSED |
| Authentication Routes | login_nonexistent_user_rejected | PASSED |

## tests/requests/questions.rs

| Category | Test | Result |
|----------|------|--------|
| Question Routes | can_get_questions | PASSED |
| Question Routes | get_questions_returns_correct_shape | PASSED |
| Question Routes | get_questions_unknown_video_returns_empty_segments | PASSED |
| Question Routes | can_update_question | PASSED |
| Question Routes | update_question_not_found | PASSED |
| Question Routes | can_update_segment_best_question | PASSED |
| Question Routes | update_segment_best_question_not_found | PASSED |

## tests/requests/answers.rs

| Category | Test | Result |
|----------|------|--------|
| Answer Routes | can_get_answers | PASSED |

## tests/requests/kids.rs

| Category | Test | Result |
|----------|------|--------|
| Kid Routes | can_get_recommendations | PASSED |
| Kid Routes | can_get_kid_tags | PASSED |
| Kid Routes | can_get_videos_assigned | PASSED |

## tests/requests/parents.rs

| Category | Test | Result |
|----------|------|--------|
| Parent Routes | can_get_parent_kids | PASSED |

## tests/requests/tags.rs

| Category | Test | Result |
|----------|------|--------|
| Tag Routes | can_get_all_tags | PASSED |

## tests/requests/videos.rs

| Category | Test | Result |
|----------|------|--------|
| Video Routes | can_get_video_tags | PASSED |

---

# Acceptance Tests

Manual acceptance tests were performed to validate end-to-end system behavior across child and parent user flows.

## Manual Tests

| Test ID | Description | Result |
|---------|-------------|--------|
| AT-01 | Mascot reads questions aloud within 3 seconds of appearance | PASSED |
| AT-02 | Video auto-pauses within ~2 seconds of distraction detection | PASSED |
| AT-03 | Parent Dashboard receives real-time distraction alert | PASSED |
| AT-04 | Correct followup question shown for correct and incorrect answers | PASSED |
| AT-05 | Results page shows per-question accuracy for a video assignment | PASSED |
| AT-06 | Results page shows detected mood per answer | PASSED |
| AT-07 | Answer processing returns a result within 1–2 seconds | PASSED |

All manual acceptance tests passed.
