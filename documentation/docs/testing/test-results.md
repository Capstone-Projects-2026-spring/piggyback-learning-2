---
sidebar_position: 4
---
# Test Results

All 16 tests passing as of 2026-03-01.

## Full Test Run Output

\```
collected 16 items                                                                                                                                                                                                              

tests/test_acceptance.py::test_student_misspells_pikachu_and_still_gets_correct PASSED                                                                                                                                    [  6%] 
tests/test_acceptance.py::test_student_answers_spinning_cat_question PASSED                                                                                                                                               [ 12%]
tests/test_integration.py::test_check_answer_correct PASSED                                                                                                                                                               [ 18%] 
tests/test_integration.py::test_get_config PASSED                                                                                                                                                                         [ 25%] 
tests/test_unit.py::test_time_to_seconds_mmss PASSED                                                                                                                                                                      [ 31%] 
tests/test_unit.py::test_time_to_seconds_hhmmss PASSED                                                                                                                                                                    [ 37%] 
tests/test_unit.py::test_time_to_seconds_bad_input PASSED                                                                                                                                                                 [ 43%] 
tests/test_unit.py::test_time_to_seconds_none PASSED                                                                                                                                                                      [ 50%] 
tests/test_unit.py::test_time_to_seconds_seconds_only PASSED                                                                                                                                                              [ 56%] 
tests/test_unit.py::test_time_to_seconds_hhmmss_full PASSED                                                                                                                                                               [ 62%] 
tests/test_unit.py::test_normalize_text_removes_stopwords PASSED                                                                                                                                                          [ 68%] 
tests/test_unit.py::test_normalize_text_maps_synonyms PASSED                                                                                                                                                              [ 75%] 
tests/test_unit.py::test_normalize_text_empty PASSED                                                                                                                                                                      [ 81%] 
tests/test_unit.py::test_build_segments_standard PASSED                                                                                                                                                                   [ 87%] 
tests/test_unit.py::test_build_segments_shorter_last PASSED                                                                                                                                                               [ 93%] 
tests/test_unit.py::test_build_segments_single PASSED                                                                                                                                                                     [100%] 

====================================================================================================== 16 passed in 2.98s ======================================================================================================
\```