---
sidebar_position: 2
---
# Integration tests

Tests to demonstrate each use-case based on the use-case descriptions and the sequence diagrams. External input should be provided via mock objects and results verified via mock objects. Integration tests should not require manual entry of data nor require manual interpretation of results.

# Tests for user/views.py

These test if user login works properly.

## 1. `test_verify_password_success_admin`
**Objective**
Verify user is able to login as admin if correct username and password was entered. 

**Results**
200 OK, redirect and success returned.
Passed

## 2. `test_verify_password_success_expert`
**Objective**
Verify user is able to login as expert if correct username and password was entered.

**Results**
200 OK, redirect and success returned
Passed

## 3. `test_verify_password_failure_invalid_password`
**Objective**
Verify user is unable to login if correct username and incorrect password was entered.

**Results**
400 BAD REQUEST and fail returned
Passed

# Tests for ai/views.py

## 4. `test_check_answer_correct_exact_match`
**Objective**
Verifies that if the user answers correctly, the program recognizes it as correct.

Example-
Expected: car
User Answer: car
Program Response: correct

**Results**
True returned
Passed

## 5. `test_check_answer_numeric_match`
**Objective**
Verifies correct numeric answers will work correctly. If user says "5", and the answer is "five" the code should process "five" as "5" thus seeing the answer as correct.

Example-
Expected: five
User Answer: 5
Program Response: correct

**Results**
200 OK and correct returned
Passed

## 6. `test_check_answer_numeric_mismatch`
**Objective**
Verifies that if user answers with incorrect numeric answers then the program should process it as an incorrect answer.

Example-
Expected: five
User Answer: one
Program Response: wrong

**Results**
200 OK and wrong returned
Passed

## 7. `test_check_answer_missing_numeric_answer`
**Objective**
If non-numeric answer given when numeric answer expected, the program will consider it wrong.

Example-
Expected: one
User Answer: many
Program Response: wrong

**Results**
200 OK, wrong and Missing numeric answer returned
Passed

## 8. `test_check_answer_correct_non_numeric`
**Objective**
If user gives a correct non-numeric answer, the program should consider it correct.

Example-
Expected: camera
User Answer: camera
Program Response: correct

**Results**
200 OK, correct, and Matched returned
Passed

## 9. `test_check_answer_list_partial_items_matched`
**Objective**
If user gives partially correct answers, the program should consider it almost correct. For example:

Example-
Expected: cat and dog
User Answer: dog
Program Response: almost

**Results**
200 OK, almost returned
Passed

## 10. `test_check_answer_missing_input`
**Objective**
Verifies if no input from user as answer then the program should count it as incorrect.

Example-
Expected: cat
User Answer: ""
Program Response: wrong

**Results**
200 OK, wrong
Passed

## 11. `test_check_answer_high_similarity`
**Objective**
Verifies if the user's answer is very similar to correct answer then it will be counted as correct. (high similarity level)

Example-
Expected: beautiful
User Answer: beautifully
Program Response: almost

**Results**
200 OK, correct
Passed

## 12. `test_check_answer_low_similarity`
**Objective**
Verifies if the user's answer is not similar to correct answer then it will be counted as incorrect.(low similarity level)

Example-
Expected: cat
User Answer: chicken
Program Response: wrong

**Results**
200 OK, wrong
Passed

## 13. `test_check_answer_borderline_similarity`
**Objective**
Check if user answer is close to correct answer then it will be countd as almost correct. 

Example-
Expected: cat
User Answer: car
Program Response: almost

**Results**
200 OK, wrong
Passed