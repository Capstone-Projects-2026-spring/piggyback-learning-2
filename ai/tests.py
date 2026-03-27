from urllib import response
from django.test import TestCase, Client
from django.urls import reverse
from rest_framework import status
from django.core.files.uploadedfile import SimpleUploadedFile
from django.urls import reverse
from rest_framework.test import APITestCase

import json
import os
import time
import unittest

from .views import (
CheckAnswerAPIView
)
#ai/tests.py tests

#Test if the answer checking works correctly.
class CheckAnswerAPITests(TestCase):
    def setUp(self):
        self.client = Client()
        self.url = reverse('check-answer')

    #Checks if exact answer returns correct status. Returns 200 and correct. 
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

    #Checks if it evaluates numeric answers correctly. Returns 200 and correct.
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

    #Checks if wrong numeric answers return wrong status. returns 200 and wrong.
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

    #Checks if non numeric answer given and it return wrong. Returns 200 and wrong.
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

    #correct non-numeric answers should return correct status. Returns 200 and correct.
    def test_check_answer_correct_non_numeric(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'apple and banana',
                    'user': 'I like apple and banana',
                    'question': 'what fruits do they like?',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'correct')
        self.assertIn('Matched', data['reason'])

    #Partially correct answers should return almost. Returns 200 and almost.
    def test_check_answer_list_partial_items_matched(self):
        response = self.client.post(
            self.url,
            data=json.dumps(
                {
                    'expected': 'apple and banana and orange',
                    'user': 'apple and banana',
                    'question': 'what fruits does the cat like?',
                }
            ),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'almost')

    #Checks if empty or missing input should return wrong status. Returns 200 and wrong.
    def test_check_answer_missing_input(self):
        response = self.client.post(
            self.url,
            data=json.dumps({'expected': '', 'user': '', 'question': ''}),
            content_type='application/json',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['status'], 'wrong')
        self.assertEqual(data['similarity'], 0.0)

    #Checks if high similarity answer score returns correct. Returns 200 and correct.
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

    #Checks if low similarity answer score returns wrong. Returns 200 and wrong.
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

    #Checks if close similarity answers return almost. Returns 200 and almost.
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
    
#These compare the speed of mood detection vs no mood detection
#These tests are skipped if API key doesn't exist.
@unittest.skipIf(not os.getenv('OPENAI_API_KEY'), "OPENAI_API_KEY environment variable not set.")    
class MoodDetectionTests(APITestCase):
    def setUp(self):
        self.url = reverse('transcribe')

    def get_audio_file(self):
        file_path = os.path.join(os.path.dirname(__file__), 'Test_audio.mp3')
        with open(file_path, 'rb') as f:
            return SimpleUploadedFile("Test_audio.mp3", f.read(), content_type="audio/mpeg")

    def test_transcribe_without_distraction(self):
        #print("\nRunning without mood detection.")
        audio_file = self.get_audio_file()
            
        start_time = time.time()
        response = self.client.post(self.url, {
            'file': audio_file,
            'analyze_distraction': 'false'
        }, format='multipart')
        end_time = time.time()

        runtime = end_time - start_time
        print(f"No mood detection took: {runtime:.3f} seconds")
        
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertTrue(response.data.get('success'))
        self.assertNotIn('analysis', response.data)

    def test_transcribe_with_distraction(self):
        #print("\nRunning with mood detection.")
        audio_file = self.get_audio_file()
        
        start_time = time.time()
        response = self.client.post(self.url, {
            'file': audio_file,
            'analyze_distraction': 'true'
        }, format='multipart')
        end_time = time.time()

        runtime = end_time - start_time
        print(f"With mood detection took:  {runtime:.3f} seconds")

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertTrue(response.data.get('success'))