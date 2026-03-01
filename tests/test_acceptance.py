from fastapi.testclient import TestClient
from main import app

client = TestClient(app)

#pokemon video
def test_student_misspells_pikachu_and_still_gets_correct():
    # Student knows the answer but misspells it slightly
    response = client.post("/api/check_answer", json={
        "expected": "pikachu",
        "user": "pikahu",
        "question": "what is the name of this yellow pokemon"
    })

   # App should still accept close enough answers
    assert response.status_code == 200
    assert response.json()["status"] == "correct"
    
#Spinning cat video
def test_student_answers_spinning_cat_question():
    # Student watches the O I I A I spinning cat video
    # and is asked what the cat is doing
    response = client.post("/api/check_answer", json={
        "expected": "spinning",
        "user": "spinning",
        "question": "what is the cat doing"
    })
    # Student gets it right
    assert response.status_code == 200
    assert response.json()["status"] == "correct"

