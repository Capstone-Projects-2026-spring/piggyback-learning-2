---
sidebar_position: 1
---
# Unit tests
You can run these tests yourself by going to the backend directory of our project in the terminal.
Run the full test suite with:

```bash
cargo test
```

To run a specific test or module:

```bash
# Run tests matching a name pattern
cargo test <test_name>

# Run tests in a specific test file
cargo test --test <test_file>
```

### Overview
#### ```These tests validate the calculations, decision making logic, and data integrity of the application. The tests use Rust's standard #[test] and #[tokio::test] attributes, using insta for snapshot management when needed.```


# Unit Test Summary

## Models

| Test | Result |
|------|--------|
| `kid_tags::test_model` | PASSED |
| `kids::kid_parent_relationship` | PASSED |
| `parents::test_model` | PASSED |
| `questions::test_model` | PASSED |
| `frames::test_model` | PASSED |
| `generated_questions::test_model` | PASSED |
| `segments::test_model` | PASSED |
| `tags::test_model` | PASSED |
| `video_assignments::test_model` | PASSED |
| `videos::test_model` | PASSED |
| `video_tags::test_model` | PASSED |

## Requests

### Auth

| Test | Description | Result |
|------|-------------|--------|
| `can_signup_kid` | Kid signup succeeds | PASSED |
| `can_signup_parent` | Parent signup succeeds | PASSED |
| `can_login_kid` | Kid login succeeds | PASSED |
| `can_login_parent` | Parent login succeeds | PASSED |
| `login_wrong_password_rejected` | Rejects bad password | PASSED |
| `login_nonexistent_user_rejected` | Rejects unknown user | PASSED |
| `kid_signup_without_parent_id_rejected` | Kid signup requires parent ID | PASSED |
| `duplicate_kid_username_rejected` | Rejects duplicate kid username | PASSED |
| `duplicate_parent_username_rejected` | Rejects duplicate parent username | PASSED |
| `invalid_role_rejected` | Rejects invalid role | PASSED |

### Answers

| Test | Description | Result |
|------|-------------|--------|
| `can_get_answers` | Fetches answers successfully | PASSED |

### Kids

| Test | Description | Result |
|------|-------------|--------|
| `can_get_recommendations` | Fetches kid recommendations | PASSED |
| `can_get_kid_tags` | Fetches tags for a kid | PASSED |
| `can_get_videos_assigned` | Fetches assigned videos | PASSED |

### Parents

| Test | Description | Result |
|------|-------------|--------|
| `can_get_parent_kids` | Fetches kids for a parent | PASSED |

### Questions

| Test | Description | Result |
|------|-------------|--------|
| `can_get_questions` | Fetches questions successfully | PASSED |
| `get_questions_returns_correct_shape` | Response shape is correct | PASSED |
| `get_questions_unknown_video_returns_empty_segments` | Unknown video yields empty segments | PASSED |
| `can_update_question` | Updates a question successfully | PASSED |
| `update_question_not_found` | Handles missing question on update | PASSED |
| `can_update_segment_best_question` | Updates best question for segment | PASSED |
| `update_segment_best_question_not_found` | Handles missing segment on update | PASSED |

### Tags

| Test | Description | Result |
|------|-------------|--------|
| `can_get_all_tags` | Fetches all tags | PASSED |

### Videos

| Test | Description | Result |
|------|-------------|--------|
| `can_get_video_tags` | Fetches tags for a video | PASSED |

---

