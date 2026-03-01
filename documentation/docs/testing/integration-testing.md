---
sidebar_position: 2
---
# Integration tests

Tests to demonstrate each use-case based on the use-case descriptions and the sequence diagrams. External input should be provided via mock objects and results verified via mock objects. Integration tests should not require manual entry of data nor require manual interpretation of results.

# Tests for user/views.py

These test if user login works properly.

## 1. `test_verify_password_success_admin`
**Objective:**
Verify user is able to login as admin if correct username and password was entered.

**Results**
200 OK, redirect and success returned.
Passed

## 2. `test_verify_password_success_expert`
**Objective:**
Verify user is able to login as expert if correct username and password was entered.

**Results**
200 OK, redirect and success returned
Passed

## 3. `test_verify_password_failure_invalid_password`
**Objective:**
Verify user is unable to login if correct username and incorrect password was entered.

**Results**
400 BAD REQUEST and fail returned
Passed

# Tests for ai/views.py

## 4