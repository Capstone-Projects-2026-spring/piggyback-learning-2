---
sidebar_position: 1
---
# Unit tests
You can run these tests yourself by running 
```python manage.py test``` 
in the terminal (in the backend directory of the project)

### Overview
#### ```These tests validate the calculations, decision making logic, and data integrity of the application. The tests use Rust's standard #[test] and #[tokio::test] attributes, using insta for snapshot management when needed.```


## ```AI Logic and Transcription Preparation (tests/utils/openai.rs)```
### Overview
Tests the preparation of data before it is sent to the AI.

### sample_frames_logic
#### Purpose
Checks if the algorithm that selects the frames to send to AI for assist it in question generation, is successful.
#### Expected Behavior
Reduces a large set of frames to a manageable amount.

### build_transcript_formatting
#### Purpose
Validates that the program correctly converts database frame and subtitles into a single formatted string.
#### Expected Behavior
Produces a transcript string with correct timestamp text formatting.

### build_prompt_structure
#### Purpose
Ensures the prompt sent to AI has all required segments and history.
#### Expected Behavior
Verified the inclusion of transcripts, duration metadata, and previously asked questions to avoid the ai repeating.

## ```Database, and Models (tests/models/)```
Overview: Validates database constraints, relationships, and automated hooks.

### ```video_update_timestamp_logic (hook.rs)```
#### Purpose
Verifies that the before_save hook automatically updates the updated_at timestamp.
#### Expected Behavior
The updated_at value after an update is strictly greater than the initial value.

### ```kid_parent_relationship (kids.rs)```
#### Purpose
Ensures relationship between parents and kids is correct.
#### Expected Behavior
A child record stays linked to the parent ID after insertion and retrieval from the database.

### ```video_assignment_json_storage (video_assignments.rs)```
#### Purpose
Validates that the JSON Answer blob is correctly stored in SQLite.
#### Expected Behavior
The JSON retrieved from the database matches the structure of the Answer struct defined in controller.

