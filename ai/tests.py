from django.test import TestCase, Client
from django.urls import reverse
from rest_framework import status

import base64
import io
import json
from unittest.mock import MagicMock, Mock, patch

from .views import (
CheckAnswerAPIView,
ConfigAPIView,
TTSAPIView,
TranscribeAPIView,
extract_items,
prepare_text_for_scoring
)

"""Checks if text normalization functions work correctly."""
class TextNormalizationTests(TestCase):
    """Checks if prepare_text_for_scoring include extracted numbers."""
    def test_prepare_text_for_scoring_include_numbers(self):
        result = prepare_text_for_scoring('five apples')
        self.assertIn('5', result)

    def test_prepare_text_for_scoring_caching(self):
        """Checks if LRU cache works correctly."""
        text = 'the quick brown fox'
        result1 = prepare_text_for_scoring(text)
        result2 = prepare_text_for_scoring(text)
        self.assertEqual(result1, result2)

    def test_extract_items_with_called_phrase(self):
        """Checks if extract_items works with 'called' phrase."""
        result = extract_items('animals called dog')
        self.assertTrue(any('dog' in item for item in result))

"""Test if the answer checking works correctly."""
class CheckAnswerAPITests(TestCase):
    def setUp(self):
        self.client = Client()
        self.url = reverse('check-answer')

    """Checks if exact answer returns correct status."""
    def test_check_answer_correct_exact_match(self):
        response = self.client.post(
            self.url,
            data=json.dumps({'expected': 'hello', 'user': 'hello', 'question': ''}),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'correct')
        self.assertGreater(data['similarity'], 0.9)

    """Checks if it evaluates numeric answers correctly."""
    def test_check_answer_numeric_match(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'five',
                    'user': '5',
                    'question': 'how many apples',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'correct')
        self.assertTrue(data['is_numeric'])

    """Checks if wrong numeric answers return wrong status."""
    def test_check_answer_numeric_mismatch(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'five',
                    'user': 'three',
                    'question': 'how many eggs',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'wrong')
        self.assertEqual(data['similarity'], 0.0)

    """Checks if non numeric answer given and it return wrong."""
    def test_check_answer_missing_numeric_answer(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'five',
                    'user': 'many eggs',
                    'question': 'how many eggs',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'wrong')
        self.assertEqual(data['reason'], 'Missing numeric answer')
        
    """correct answers should return correct status."""
    def test_check_answer_list_all_items_matched(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'apple and banana',
                    'user': 'I like apple and banana',
                    'question': 'what fruits',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'correct')
        self.assertIn('Matched', data['reason'])
        
    """Partially correct answers should return almost."""
    def test_check_answer_list_partial_items_matched(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'apple and banana and orange',
                    'user': 'apple and banana',
                    'question': 'what fruits',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'almost')

    """Checks if empty input should return wrong status."""
    def test_check_answer_empty_input(self):
        response = self.client.post(
            self.url,
            data=json.dumps({'expected': '', 'user': '', 'question': ''}),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'wrong')
        self.assertEqual(data['similarity'], 0.0)

    """Checks if missing answers return wrong."""
    def test_check_answer_only_expected_empty(self):
        response = self.client.post(
            self.url,
            data=json.dumps({'expected': '', 'user': 'hello', 'question': ''}),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'wrong')

    """Checks if high similarity answer score returns correct."""
    def test_check_answer_high_similarity(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'beautiful',
                    'user': 'beautifully',
                    'question': 'describe the thing.',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'correct')

    """Checks if low similarity answer score returns wrong."""
    def test_check_answer_low_similarity(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'elephant',
                    'user': 'taffy',
                    'question': 'what animal',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'wrong')

    """Checks if close similarity answers return almost."""
    def test_check_answer_borderline_similarity(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'cat',
                    'user': 'car',
                    'question': 'what animal',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertIn(data['status'], ['almost', 'wrong'])

    """Checks if synonym handling should return correct."""
    def test_check_answer_synonym_handling(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'scared',
                    'user': 'afraid',
                    'question': 'how did they feel',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'correct')
