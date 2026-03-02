---
sidebar_position: 1
---
# Unit tests

All unit test are in `tests/test_unit.py` and uses pytest

## Overview
These tests verify the core helper functions used throughout the app. Each function is tested in isolation with no external dependencies - no database, no API calls, no network.

Functions tested:
- `time_to_seconds` - used to convert video timestamps into seconds for segment calculations
- `normalize_text` -used to clean and normalize student answers before grading
- `build_segments_from_duration` - used to split a video into timed quiz segments

## time_to_seconds 
Converts a time string to total seconds.

**Details:** Tested in isolation with no external dependencies. All external classes are stubbed.
| Test Case | Input | Expected Result |
|---|---|---|
| MM:SS format | `"1:30"` | `90` |
| HH:MM:SS format | `"1:00:00"` | `3600` |
| Invalid string | `"bad"` | `0` |
| None input | `None` | `AttributeError` |
| Seconds only | `"45"` | `45` |
| Full HH:MM:SS | `"2:30:15"` | `9015` |

## normalize_text
Removes stopwords and map synonyms for a test string 
like the big dog  can just be big dog

**Details:** Tested in isolation with no external dependencies. All external classes are stubbed.

| Test Case | Input | Expected Result |
|---|---|---|
| Removes stopwords | `"the big dog"` | `"big dog"` |
| Maps synonyms | `"scared"` | `"afraid"` |
| Empty string | `""` | `""` |

## build_segments_from_duration
Splits a video duration into timed segments for quiz question placement.

**Details:** Tested in isolation with no external dependencies. All external classes are stubbed.

| Test Case | Input | Expected Result |
|---|---|---|
| Standard segments | `(180, 60)` | `[(0,59),(60,119),(120,179),(180,180)]` |
| Shorter last segment | `(90, 60)` | `[(0,59),(60,90)]` |
| Single segment | `(60, 60)` | `[(0,59),(60,60)]` |
