from fastapi.testclient import TestClient
from main import app

#warps the app into the fkae browser , client can send GET ,POST etc request to it.
client = TestClient(app)

def test_check_answer_correct():
    #POST request to like our frontend.
    response = client.post("/api/check_answer", json= {
        "expected": "dog",
        "user" : "dog",
        "question": "what animal is it" 
    })
    #did the server respond well? 
    assert response.status_code == 200
    assert response.json()["status"]== "correct"

    #Get config which returns app config
    #checking if the route exists, if its deleted or waht not.
    
def test_get_config():
    #GET request to that endpoint
    response = client.get("/api/config")
    assert response.status_code == 200
    #threshold comfirms that the forntend will get data it needs to work.
    assert "thresholds" in response.json()