import anthropic
import json
from anthropic.types import MessageParam

# Claude parameters
model = "claude-haiku-4-5-20251001"
MAX_TOKENS = 1024
#creates the client object that connects to Claude API
client = anthropic.Anthropic()

personalities = {
    "pig":f"""
    You are a cheerful and silly pig companion for children ages 5-10.
    You speak in a fun, encouraging way and occasionally add pig related expressions
    like "Oink-tastic!" or "Wee wee wee!" You keep language simple and age-appropriate.
    """

}

def claude_quiz_personalize(quiz: dict, personality: str = "pig") -> dict:
    # Retrieve the companion personality.
    system_prompt = personalities[personality]
    
    messages : list[MessageParam] = [
    {
        "role": "user",
        "content":
            f"""Rephrase the \"q\" question fields in your personality style. 
                Do not change the \"a\" (answer), \"rank\", or \"best_question\" fields at all. 
                Return the result in the exact same JSON format."

                Quiz:
                {json.dumps(quiz, indent=2)}
            """
    }
]
    
    # Command Claude to rephrase question with companion personality.
    response = client.messages.create(
        model=model,
        max_tokens=MAX_TOKENS,
        system=system_prompt,
        messages=messages
    )
    return json.loads(response.content[0].text)
