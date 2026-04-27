---
sidebar_position: 2
---
# Integration tests

Tests to demonstrate each use-case based on the use-case descriptions and the sequence diagrams. External input should be provided via mock objects and results verified via mock objects. Integration tests should not require manual entry of data nor require manual interpretation of results.

## Overview
These tests validates :
- The Axum routes interact correctly with the SQLite database
- The expected status codes were returned
- The expected JSON structures were returned.
The tests use the loco-rs testing library, tokio::test to simulate asynchronous requests, and serial_test to keep the database state consistent.

### ```Authentication Tests (tests/requests/auth.rs)```
#### 1. can_signup_parent
##### Purpose
Verifies that a new parent can register with valid login credentials.
##### Expected Behavior
Responds with status code 200 and success set to true.

#### 2. can_signup_kid
##### Purpose
Ensures child accounts can be created when they are linked to a valid parent in the DB.
##### Expected Behavior
Parent is successfully queried in the database, and registering a child returns 200.

#### 3. duplicate_parent_username_rejected
##### Purpose
To prevent multiple parent accounts from having the same username.
##### Expected Behavior
First request returns a 200 status code. second request with same payload returns 400 status code.

#### 4. duplicate_kid_username_rejected
##### Purpose
Prevents multiple child accounts from having the same username.
##### Expected Behavior
The system rejects an attempt at the second signup using the same username with a 400 status code.

#### 5. kid_signup_without_parent_id_rejected
##### Purpose
To ensure the functionality of the requirement that kids accounts must be connected to a parent account.
##### Expected Behavior
Request fails with 400 status code if the parent_id is missing.

#### 6. invalid_role_rejected
##### Purpose
Ensures that users can only register with specified roles. For security reasons.
##### Expected Behavior
Attempting to register with a role other than "parent" or "kid" returns 400 status code.

#### 7. can_login_parent
##### Purpose
To verify that the parent can login and JWT generation.
##### Expected Behavior
Returns 200, a valid token, and the correct user role.

#### 8. can_login_kid
##### Purpose 
Verifies kid login and the parent_username is included in the response.
##### Expected Behavio
Returns 200, a valid token, and the linked parent's username.

#### 9. login_wrong_password_rejected
##### Purpose
Ensures the authentication fails for incorrect passwords. Users can't login if they enter the wrong password.
##### Expected Behavior
Responds with status code 400.

#### 10.login_nonexistent_user_rejected
##### Purpose
Ensures login fails for users not present in the database.
##### Expected Behavior
Responds with status code 400.


### ```Questions (tests/requests/questions.rs)```
#### 1. can_get_questions
##### Purpose
Verifies questions can be retrieved for a video segment.
##### Expected Behavior
Responds with 200 status code and seeded question data.

#### 2. get_questions_returns_correct_shape
##### Purpose
Validates the JSON schema for the quiz player.
##### Expected Behavior
Response includes video_id and a segments array.

#### 3. get_questions_unknown_video_returns_empty_segments
##### Purpose
Ensures the application correctly handles queries for missing video data.
##### Expected Behavior
Returns 200 with an empty list for segments.

#### 4. can_update_question
##### Purpose
To verify that existing questions can be edited by the parent.
##### Expected Behavior 
PATCH request returns 200.

#### 5. update_question_not_found
##### Purpose
Ensures correct behavior when trying to update a question that doesn't exist.
##### Expected Behavior
Responds with status code 404.

#### 6. can_update_segment_best_question
##### Purpose
Verifies update logic is correct for the "best question" field.
##### Expected Behavior
Responds with status code 200.

##### update_segment_best_question_not_found
##### Purpose
Ensures correct behavior when updating a missing segment.
##### Expected Behavior
Responds with status code 404.

## ```User Interactions (tests/requests/kids.rs, parents.rs, tags.rs, videos.rs, answers.rs)```
#### 1. can_get_answers
##### Purpose
To check if a child's quiz performance history for a specific video can be retrieved.
##### Expected Behavior
Returns status 200 and available JSON answer data.

#### 2. can_get_recommendations
##### Purpose
Validates that the recommendation system with the tags function as expected.
##### Expected Behavior
Responds with status code 200.

#### 3. can_get_kid_tags / can_get_video_tags
##### Purpose
To verify that specific tags linked to kid accounts or videos are successfully retrieved.
##### Expected Behavior
Responds with status code 200.

#### 4. can_get_videos_assigned
##### Purpose
To verify that videos assigned to a child can be retrieved.
##### Expected Behavior
Responds with status code 200.

#### 5. can_get_parent_kids
##### Purpose
To see if application can list all children associated with a parent ID.
##### Expected Behavior
Responds with status code 200.

#### 6. can_get_all_tags
##### Purpose
Verifies that the global tag library is available.
##### Expected Behavior
Responds with status code 200.