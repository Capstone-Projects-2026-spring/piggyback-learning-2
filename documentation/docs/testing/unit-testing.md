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
| `kid_tags::test_model` | ✅ ok |
| `kids::kid_parent_relationship` | ✅ ok |
| `parents::test_model` | ✅ ok |
| `questions::test_model` | ✅ ok |
| `frames::test_model` | ✅ ok |
| `generated_questions::test_model` | ✅ ok |
| `segments::test_model` | ✅ ok |
| `tags::test_model` | ✅ ok |
| `video_assignments::test_model` | ✅ ok |
| `videos::test_model` | ✅ ok |
| `video_tags::test_model` | ✅ ok |

## Requests

### Auth

| Test | Description | Result |
|------|-------------|--------|
| `can_signup_kid` | Kid signup succeeds | ✅ ok |
| `can_signup_parent` | Parent signup succeeds | ✅ ok |
| `can_login_kid` | Kid login succeeds | ✅ ok |
| `can_login_parent` | Parent login succeeds | ✅ ok |
| `login_wrong_password_rejected` | Rejects bad password | ✅ ok |
| `login_nonexistent_user_rejected` | Rejects unknown user | ✅ ok |
| `kid_signup_without_parent_id_rejected` | Kid signup requires parent ID | ✅ ok |
| `duplicate_kid_username_rejected` | Rejects duplicate kid username | ✅ ok |
| `duplicate_parent_username_rejected` | Rejects duplicate parent username | ✅ ok |
| `invalid_role_rejected` | Rejects invalid role | ✅ ok |

### Answers

| Test | Description | Result |
|------|-------------|--------|
| `can_get_answers` | Fetches answers successfully | ✅ ok |

### Kids

| Test | Description | Result |
|------|-------------|--------|
| `can_get_recommendations` | Fetches kid recommendations | ✅ ok |
| `can_get_kid_tags` | Fetches tags for a kid | ✅ ok |
| `can_get_videos_assigned` | Fetches assigned videos | ✅ ok |

### Parents

| Test | Description | Result |
|------|-------------|--------|
| `can_get_parent_kids` | Fetches kids for a parent | ✅ ok |

### Questions

| Test | Description | Result |
|------|-------------|--------|
| `can_get_questions` | Fetches questions successfully | ✅ ok |
| `get_questions_returns_correct_shape` | Response shape is correct | ✅ ok |
| `get_questions_unknown_video_returns_empty_segments` | Unknown video yields empty segments | ✅ ok |
| `can_update_question` | Updates a question successfully | ✅ ok |
| `update_question_not_found` | Handles missing question on update | ✅ ok |
| `can_update_segment_best_question` | Updates best question for segment | ✅ ok |
| `update_segment_best_question_not_found` | Handles missing segment on update | ✅ ok |

### Tags

| Test | Description | Result |
|------|-------------|--------|
| `can_get_all_tags` | Fetches all tags | ✅ ok |

### Videos

| Test | Description | Result |
|------|-------------|--------|
| `can_get_video_tags` | Fetches tags for a video | ✅ ok |

---

**Total: 35 passed, 0 failed** — finished in 8.11s
