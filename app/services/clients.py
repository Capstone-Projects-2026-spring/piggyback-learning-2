import os
from functools import lru_cache

import google.generativeai as genai
from openai import OpenAI

from app.settings import GEMINI_API_KEY


@lru_cache(maxsize=1)
def get_openai_client() -> OpenAI:
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key or not api_key.strip():
        raise RuntimeError("Missing OPENAI_API_KEY in environment (.env / .env.txt).")
    return OpenAI(api_key=api_key)


OPENAI_CLIENT = get_openai_client()

if GEMINI_API_KEY:
    genai.configure(api_key=GEMINI_API_KEY)

