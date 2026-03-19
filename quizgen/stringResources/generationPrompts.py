# gemini prompts for quiz generation stored here (used in generation.py).
# some prompts are stored in functions rather than constants because they 
# require dynamic input (e.g. transcript, duration, etc.)

class GenerationPrompts:
    SYSTEM_MESSAGE = (
        'You are a safe, child-focused educational assistant. '
        "The content is a children's educational video. "
        'Follow all safety policies and avoid disallowed content. '
        'Provide age-appropriate, neutral, factual responses only.'
    )

    def get_generation_prompt(transcript: str, duration: int, start_time: int, end_time: int) -> str:
        base_prompt = f"""You are an early childhood educator designing comprehension questions for children ages 6–8.
            Analyze the video content using both the visual frames and the complete transcript provided below.

            COMPLETE TRANSCRIPT:
            ==========================================
            {transcript}
            ==========================================

            TASK:
            I am providing you with sequential frames from a {duration}-second segment ({start_time}s to {end_time}s).

            1. Provide ONE short, child-friendly comprehension question for EACH category:
            - Character
            - Setting
            - Feeling
            - Action
            - Causal Relationship
            - Outcome
            - Prediction

            2. Rank the questions (best = 1)

            3. Return JSON only in this structure:
            {{
            "questions": {{
                "character": {{ "q": "...", "a": "...", "rank": "" }},
                "setting": {{ "q": "...", "a": "...", "rank": "" }},
                "feeling": {{ "q": "...", "a": "...", "rank": "" }},
                "action": {{ "q": "...", "a": "...", "rank": "" }},
                "causal": {{ "q": "...", "a": "...", "rank": "" }},
                "outcome": {{ "q": "...", "a": "...", "rank": "" }},
                "prediction": {{ "q": "...", "a": "...", "rank": "" }}
            }},
            "best_question": "..."
            }}
            """
        return base_prompt
    
    def get_polite_generation_prompt(transcript: str, duration: int, start_time: int, end_time: int) -> str:
        polite_prompt = f"""You are helping create educational questions for young children. This is a children's educational video with no violence or inappropriate content.
            COMPLETE TRANSCRIPT:
            ==========================================
            {transcript}
            ==========================================

            I am providing you with sequential frames from a {duration}-second segment ({start_time}s to {end_time}s).

            Create ONE short, child-friendly comprehension question for EACH category:
            - Character
            - Setting
            - Feeling
            - Action
            - Causal Relationship
            - Outcome
            - Prediction

            Rank the questions (best = 1)

            Return JSON only in this structure:
            {{
            "questions": {{
                "character": {{ "q": "...", "a": "...", "rank": "" }},
                "setting": {{ "q": "...", "a": "...", "rank": "" }},
                "feeling": {{ "q": "...", "a": "...", "rank": "" }},
                "action": {{ "q": "...", "a": "...", "rank": "" }},
                "causal": {{ "q": "...", "a": "...", "rank": "" }},
                "outcome": {{ "q": "...", "a": "...", "rank": "" }},
                "prediction": {{ "q": "...", "a": "...", "rank": "" }}
            }},
            "best_question": "..."
            }}
            """
        return polite_prompt