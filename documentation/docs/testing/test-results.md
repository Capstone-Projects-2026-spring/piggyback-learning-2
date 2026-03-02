---
sidebar_position: 4
---
# Test Results

All 16 tests passing as of 2026-03-01.

## Full Test Run Output

\```
collected 16 items

tests/test_acceptance.py::test_student_loads_app_and_answers_correctly PASSED
tests/test_acceptance.py::test_student_misspells_pikachu_and_still_gets_correct PASSED
tests/test_acceptance.py::test_student_answers_spinning_cat_question PASSED
tests/test_integration.py::test_check_answer_correct PASSED
tests/test_integration.py::test_get_config PASSED
tests/test_unit.py::test_time_to_seconds_mmss PASSED
tests/test_unit.py::test_time_to_seconds_hhmmss PASSED
tests/test_unit.py::test_time_to_seconds_bad_input PASSED
tests/test_unit.py::test_time_to_seconds_none PASSED
tests/test_unit.py::test_time_to_seconds_seconds_only PASSED
tests/test_unit.py::test_time_to_seconds_hhmmss_full PASSED
tests/test_unit.py::test_normalize_text_removes_stopwords PASSED
tests/test_unit.py::test_normalize_text_maps_synonyms PASSED
tests/test_unit.py::test_normalize_text_empty PASSED
tests/test_unit.py::test_build_segments_standard PASSED
tests/test_unit.py::test_build_segments_single PASSED

16 passed in 2.98s

\```