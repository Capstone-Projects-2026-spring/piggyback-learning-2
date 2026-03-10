from app.services.question_generation_service import time_to_seconds
import pytest
from video_quiz_routes import normalize_text
from app.services.question_generation_service import build_segments_from_duration
#pytest runs any function starting with test_ 
# Testing assignment service functions
from app.services.sqlite_store import init_db
from app.services.expert_auth_service import (
    add_video_assignment,
    remove_video_assignment,
    list_experts_for_video,
    can_expert_access_video,
    claim_video_for_expert,
    create_expert,
)
#make a value to test
def setup_module():
    init_db()
    try:
        create_expert("testexpert1", "Test Expert 1", "password123")
        create_expert("testexpert2", "Test Expert 2", "password123")
    except:
        pass 
#Testing time_to_seconds
def test_time_to_seconds_mmss():
    assert time_to_seconds("1:30") == 90

def test_time_to_seconds_hhmmss():
    assert time_to_seconds("1:00:00") == 3600

def test_time_to_seconds_bad_input():
    assert time_to_seconds("bad") == 0

def test_time_to_seconds_none():
    with pytest.raises(AttributeError):
            time_to_seconds(None) == 0

def test_time_to_seconds_seconds_only():
    assert time_to_seconds("45") == 45

def test_time_to_seconds_hhmmss_full():
    assert time_to_seconds("2:30:15") == 9015


#Testing for normalize_text
def test_normalize_text_removes_stopwords():
     assert normalize_text("the big dog") == "big dog"
    
def test_normalize_text_maps_synonyms():
    assert normalize_text("scared") == "afraid"

def test_normalize_text_empty():
    assert normalize_text("") == ""

#Testing for build_segment_from_duration
def test_build_segments_standard():
    assert build_segments_from_duration(180, 60) == [(0, 59), (60, 119), (120, 179), (180, 180)]

def test_build_segments_shorter_last():
    assert build_segments_from_duration(90, 60) == [(0, 59), (60, 90)]

def test_build_segments_single():
    assert build_segments_from_duration(60, 60) == [(0, 59), (60, 60)]

#Testing services
def test_add_assignment():
    add_video_assignment("vid_test", "testexpert1")
    assert can_expert_access_video("testexpert1", "vid_test") == True

def test_two_experts_same_video():
    add_video_assignment("vid_test", "testexpert2")
    experts = list_experts_for_video("vid_test")
    assert len(experts) == 2


def test_remove_assignment():
    remove_video_assignment("vid_test", "testexpert1")
    assert can_expert_access_video("testexpert1", "vid_test") == False

def test_claim_is_idempotent():
    claim_video_for_expert("testexpert2", "vid_test")
    claim_video_for_expert("testexpert2", "vid_test")  # twice, no error
    assert can_expert_access_video("testexpert2", "vid_test") == True