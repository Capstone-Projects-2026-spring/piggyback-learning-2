import base64
import io
import os
import re
from functools import lru_cache
import time

from openai import OpenAI
from rapidfuzz import fuzz
from rest_framework import status
from rest_framework.parsers import JSONParser
from rest_framework.response import Response
from rest_framework.views import APIView

GRADING_CONFIG = {
    'rapidfuzz_correct': 0.82,
    'rapidfuzz_wrong': 0.55,
    'use_ai': True,
    'ai_model': 'gpt-4o-mini',
    'ai_temperature': 0.0,
    'ai_max_tokens': 4,
    'ai_timeout': 10,
}

DISTRACTION_CONFIG = {
    'pause_threshold_seconds': 0.5,      # gaps longer than 0.5 secs = pause
    'filler_ratio_threshold': 0.2,        # greater than 20% filler words = distracted
    'max_pause_count': 3,                  # more than 3 pauses = distracted
    'min_words_for_analysis': 5,           # short responses = ignored (short responses don't count)
}

NUM_WORDS = {
    'zero': 0,
    'one': 1,
    'two': 2,
    'three': 3,
    'four': 4,
    'five': 5,
    'six': 6,
    'seven': 7,
    'eight': 8,
    'nine': 9,
    'ten': 10,
    'eleven': 11,
    'twelve': 12,
    'thirteen': 13,
    'fourteen': 14,
    'fifteen': 15,
    'sixteen': 16,
    'seventeen': 17,
    'eighteen': 18,
    'nineteen': 19,
    'twenty': 20,
}

SCALE_WORDS = {'hundred': 100, 'thousand': 1000, 'million': 1_000_000}

STOPWORDS = {
    'the',
    'a',
    'an',
    'is',
    'are',
    'and',
    'of',
    'to',
    'it',
    'in',
    'on',
    'at',
    'for',
    'was',
    'were',
    'be',
    'being',
    'been',
    'am',
    'do',
    'did',
    'does',
    'done',
    'they',
    'them',
    'their',
    'there',
    'here',
    'that',
    'this',
    'these',
    'those',
    'i',
    'you',
    'he',
    'she',
    'we',
    'me',
    'my',
    'your',
    'his',
    'her',
    'our',
    'ours',
    'with',
    'by',
    'from',
}

FILLER_WORDS = {'um', 'uh', 'like', 'you know', 'hmm', 'well', 'okay', 'so'}

SINGLE_FILLERS = {w for w in FILLER_WORDS if ' ' not in w}

SYNONYMS = {
    'scared': 'afraid',
    'frightened': 'afraid',
    'fearful': 'afraid',
    'nervous': 'afraid',
    'worried': 'afraid',
    'sad': 'unhappy',
    'crying': 'unhappy',
    'mad': 'angry',
    'upset': 'angry',
    'annoyed': 'angry',
    'mom': 'mother',
    'mommy': 'mother',
    'dad': 'father',
    'daddy': 'father',
    'grandma': 'grandmother',
    'grandpa': 'grandfather',
    'bro': 'brother',
    'sis': 'sister',
    'sissy': 'sister',
    'puppy': 'dog',
    'puppies': 'dog',
    'kitten': 'cat',
    'kitties': 'cat',
    'bunny': 'rabbit',
    'hare': 'rabbit',
    'pony': 'horse',
    'soda': 'drink',
    'juice': 'drink',
    'milk': 'drink',
    'water': 'drink',
    'snack': 'food',
    'meal': 'food',
    'candy': 'sweet',
    'sweets': 'sweet',
    'chocolate': 'sweet',
    'cookie': 'sweet',
    'icecream': 'sweet',
    'ice cream': 'sweet',
    'cake': 'sweet',
    'pie': 'sweet',
    'automobile': 'car',
    'truck': 'car',
    'bus': 'car',
    'bike': 'bicycle',
    'tv': 'television',
    'show': 'movie',
    'cartoon': 'movie',
    'film': 'movie',
    'large': 'big',
    'huge': 'big',
    'giant': 'big',
    'enormous': 'big',
    'little': 'small',
    'tiny': 'small',
    'short': 'small',
    'quick': 'fast',
    'speedy': 'fast',
    'yeah': 'yes',
    'yep': 'yes',
    'yup': 'yes',
    'nope': 'no',
    'nah': 'no',
}


def get_openai_client() -> OpenAI:
    api_key = os.getenv('OPENAI_API_KEY')
    if not api_key or not api_key.strip():
        raise RuntimeError('Missing OPENAI_API_KEY')
    return OpenAI(api_key=api_key)


def words_to_numbers(text: str) -> list[int]:
    text = (text or '').lower().strip()
    numbers = [int(d) for d in re.findall(r'\d+', text)]
    tokens = re.split(r'[-\s]+', text)
    total, current, found_number = 0, 0, False

    for token in tokens + ['end']:
        if token in NUM_WORDS:
            found_number = True
            current += NUM_WORDS[token]
        elif token in SCALE_WORDS:
            found_number = True
            scale = SCALE_WORDS[token]
            if current == 0:
                current = 1
            current *= scale
            if scale > 100:
                total += current
                current = 0
        else:
            if found_number:
                total += current
                numbers.append(total)
                total, current, found_number = 0, 0, False

    return numbers


def normalize_text(text: str) -> str:
    tokens = re.findall(r'[a-z]+', (text or '').lower())
    normalized = []
    for t in tokens:
        if t in STOPWORDS or t in FILLER_WORDS or t in NUM_WORDS or t in SCALE_WORDS:
            continue
        normalized.append(SYNONYMS.get(t, t))
    return ' '.join(normalized)


@lru_cache(maxsize=2048)
def prepare_text_for_scoring(text: str) -> str:
    if not text:
        return ''
    normalized = normalize_text(text)
    number_tokens = words_to_numbers(text)
    if number_tokens:
        unique_numbers = ' '.join(str(n) for n in sorted(set(number_tokens)))
        return f'{normalized} {unique_numbers}'.strip()
    return normalized


def extract_items(expected_raw: str) -> list[str]:
    parts = [
        p
        for p in re.split(r',|\sand\s', expected_raw, flags=re.IGNORECASE)
        if p.strip()
    ]
    out = []
    for p in parts:
        p = p.strip()
        m = re.search(r'\bcalled\s+(.+)', p)
        if m:
            p = m.group(1)
        norm = normalize_text(p)
        toks = norm.split()
        if len(toks) > 3:
            norm = ' '.join(toks[-3:])
        if norm:
            out.append(norm)
    return out


def list_match(expected_raw: str, user_raw: str) -> tuple[int, int]:
    items = extract_items(expected_raw)
    user_norm = normalize_text(user_raw)
    matched = set()
    for item in items:
        score = max(
            fuzz.partial_ratio(item, user_norm), fuzz.token_set_ratio(item, user_norm)
        )
        if score >= 60:
            matched.add(item)
    return len(matched), len(items)


class CheckAnswerAPIView(APIView):
    authentication_classes = []
    permission_classes = []

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        expected = (payload.get('expected') or '').strip().lower()
        user = (payload.get('user') or '').strip().lower()
        question = (payload.get('question') or '').strip().lower()

        if not expected or not user:
            return Response(
                {
                    'similarity': 0.0,
                    'expected': expected,
                    'user': user,
                    'is_numeric': False,
                    'status': 'wrong',
                    'reason': 'Empty input',
                }
            )

        expected_numbers = words_to_numbers(expected)
        user_numbers = words_to_numbers(user)
        is_numeric = bool(expected_numbers)
        numeric_question = bool(
            re.search(r'\bhow many\b|\bnumber of\b|\bhow much\b|\bcount\b', question)
        )
        expected_text = normalize_text(expected)

        if expected_numbers:
            expected_set = set(expected_numbers)
            user_set = set(user_numbers)
            if user_numbers and not (expected_set & user_set):
                return Response(
                    {
                        'similarity': 0.0,
                        'expected': expected,
                        'user': user,
                        'is_numeric': True,
                        'status': 'wrong',
                        'reason': 'Numeric mismatch',
                    }
                )
            if user_numbers and not expected_text and expected_set == user_set:
                return Response(
                    {
                        'similarity': 1.0,
                        'expected': expected,
                        'user': user,
                        'is_numeric': True,
                        'status': 'correct',
                        'reason': 'Numeric answer matched',
                    }
                )
            if not user_numbers and (not expected_text or numeric_question):
                return Response(
                    {
                        'similarity': 0.0,
                        'expected': expected,
                        'user': user,
                        'is_numeric': True,
                        'status': 'wrong',
                        'reason': 'Missing numeric answer',
                    }
                )

        exp_clean = prepare_text_for_scoring(expected)
        usr_clean = prepare_text_for_scoring(user)

        pr = fuzz.partial_ratio(exp_clean, usr_clean) / 100.0
        tsr = fuzz.token_set_ratio(exp_clean, usr_clean) / 100.0
        score = max(pr, tsr)

        items = extract_items(expected)
        if len(items) > 1:
            matched_count, total_count = list_match(expected, user)
            if total_count > 0:
                if matched_count >= total_count:
                    return Response(
                        {
                            'similarity': round(score, 3),
                            'expected': expected,
                            'user': user,
                            'is_numeric': is_numeric,
                            'status': 'correct',
                            'reason': f'Matched {matched_count} of {total_count} items',
                        }
                    )
                if matched_count > 0:
                    return Response(
                        {
                            'similarity': round(score, 3),
                            'expected': expected,
                            'user': user,
                            'is_numeric': is_numeric,
                            'status': 'almost',
                            'reason': f'Matched {matched_count} of {total_count} items',
                        }
                    )

        if score >= GRADING_CONFIG['rapidfuzz_correct']:
            return Response(
                {
                    'similarity': round(score, 3),
                    'expected': expected,
                    'user': user,
                    'is_numeric': is_numeric,
                    'status': 'correct',
                    'reason': f'High RapidFuzz score {score:.2f}',
                }
            )

        if score <= GRADING_CONFIG['rapidfuzz_wrong']:
            return Response(
                {
                    'similarity': round(score, 3),
                    'expected': expected,
                    'user': user,
                    'is_numeric': is_numeric,
                    'status': 'wrong',
                    'reason': f'Low RapidFuzz score {score:.2f}',
                }
            )

        return Response(
            {
                'similarity': round(score, 3),
                'expected': expected,
                'user': user,
                'is_numeric': is_numeric,
                'status': 'almost',
                'reason': f'Borderline case defaulted (RapidFuzz={score:.2f})',
            }
        )

#note, time function import only exists for testing purposes, remove later if desired
class TranscribeAPIView(APIView):
    authentication_classes = []
    permission_classes = []

    def post(self, request):
        """
        POST /api/transcribe
        multipart/form-data: file=<audio>
        Optional: analyze_distraction=true (form field) #(basically the on/off button for "mood detection")
        """
        audio_file = request.FILES.get('file')
        if not audio_file:
            return Response(
                {'success': False, 'error': 'Missing file'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        #analyze_distraction = request.POST.get('analyze_distraction', '').lower() == 'true'
        analyze_distraction = True #Force true or false for testing, above is input if you want to turn it on and off from frontend
        try:
            client = get_openai_client()
            audio_bytes = io.BytesIO(audio_file.read())

            transcribe_args = {
                'model': 'whisper-1',
                'file': (audio_file.name or 'speech.webm', audio_bytes, audio_file.content_type),
            }
            start_time = time.time()

            #these arguements allow getting the timestamps of words spoken
            if analyze_distraction:
                transcribe_args['timestamp_granularities'] = ['word']
                transcribe_args['response_format'] = 'verbose_json'

            transcription = client.audio.transcriptions.create(**transcribe_args)

            response_data = {'success': True, 'text': transcription.text}

            if analyze_distraction and hasattr(transcription, 'words') and transcription.words:
                words = transcription.words
                total_words = len(words)

                #simple dictionary comparison for filler words
                filler_count = sum(1 for w in words if w.word.lower() in SINGLE_FILLERS)

                pauses = []
                #pause detection
                for i in range(1, len(words)):
                    gap = words[i].start - words[i-1].end
                    if gap > DISTRACTION_CONFIG['pause_threshold_seconds']:
                        pauses.append({
                            'start': words[i-1].end,
                            'end': words[i].start,
                            'duration': gap,
                        })
                pause_count = len(pauses)

                if total_words < DISTRACTION_CONFIG['min_words_for_analysis']:
                    distracted = False
                else:
                    filler_ratio = filler_count / total_words
                    distracted = (
                        filler_ratio > DISTRACTION_CONFIG['filler_ratio_threshold'] or
                        pause_count > DISTRACTION_CONFIG['max_pause_count']
                    )

                response_data['analysis'] = {
                    'total_words': total_words,
                    'filler_words': filler_count,
                    'filler_ratio': round(filler_count / total_words, 3) if total_words else 0,
                    'pause_count': pause_count,
                    'pauses': pauses,  
                    'distracted': distracted,
                }

                total_time = time.time() - start_time
                print(f"TranscribeAPIView time of mood detection: {total_time:.3f} seconds. total_words: {total_words}. filler_words: {filler_count}. filler_ratio: {round(filler_count / total_words, 3) if total_words else 0}. pause_count: {pause_count}. distracted: {distracted}")

            if not analyze_distraction:
                total_time = time.time() - start_time
                print(f"Transcription time no mood detection: {total_time:.3f} seconds")

            return Response(response_data)

        except Exception as e:
            return Response(
                {'success': False, 'error': str(e)},
                status=status.HTTP_500_INTERNAL_SERVER_ERROR,
            )


class ConfigAPIView(APIView):
    authentication_classes = []
    permission_classes = []

    def get(self, request):
        return Response({'skip_prevention': False, 'thresholds': GRADING_CONFIG})


class TTSAPIView(APIView):
    """
    POST /api/tts
    FastAPI takes JSON payload: {text, voice="sage", speed=0.75, format="mp3"}
    Returns: {success: True, audio: <base64>, format: "...", voice: "..."}
    """

    authentication_classes = []
    permission_classes = []
    parser_classes = [JSONParser]

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}

        text = str(payload.get('text') or '').strip()
        if not text:
            return Response(
                {'success': False, 'message': 'text is required'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        voice = str(payload.get('voice') or 'sage').strip() or 'sage'

        raw_speed = payload.get('speed', 0.75)
        try:
            speed = float(raw_speed)
        except (TypeError, ValueError):
            speed = 0.75
        speed = max(0.25, min(speed, 4.0))

        response_format = str(payload.get('format') or 'mp3').strip() or 'mp3'

        def _synthesize(voice_name: str) -> bytes:
            client = get_openai_client()
            # Mirrors FastAPI: audio.speech.with_streaming_response.create(...).read()
            with client.audio.speech.with_streaming_response.create(
                model='gpt-4o-mini-tts',
                voice=voice_name,
                input=text,
                speed=speed,
            ) as resp:
                return resp.read()

        try:
            audio_bytes = _synthesize(voice)
        except Exception as exc:
            # Match FastAPI fallback behaviour: try "alloy" if voice error-ish
            fallback_voice = 'alloy'
            error_message = str(exc)
            should_retry_with_fallback = voice.lower() != fallback_voice and any(
                kw in error_message.lower()
                for kw in ('voice', 'unknown', 'not found', 'unsupported')
            )
            if should_retry_with_fallback:
                try:
                    audio_bytes = _synthesize(fallback_voice)
                    voice = fallback_voice
                except Exception as retry_exc:
                    return Response(
                        {
                            'success': False,
                            'message': f'TTS generation failed: {error_message} | fallback_failed={retry_exc}',
                        },
                        status=status.HTTP_502_BAD_GATEWAY,
                    )
            else:
                return Response(
                    {
                        'success': False,
                        'message': f'TTS generation failed: {error_message}',
                    },
                    status=status.HTTP_502_BAD_GATEWAY,
                )

        audio_b64 = base64.b64encode(audio_bytes).decode('utf-8')
        return Response(
            {
                'success': True,
                'audio': audio_b64,
                'format': response_format,
                'voice': voice,
            }
        )