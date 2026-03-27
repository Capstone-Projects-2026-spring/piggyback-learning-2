from django.test import TestCase
from django.urls import reverse
from rest_framework.test import APIClient
from rest_framework import status

from videos.models import Video
from quizgen.models import SubmittedQuestionSet

from review.models import (
    ExpertAnnotation,
    ExpertQuestion,
    FinalQuestionSet,
    FinalSegment,
    FinalAIQuestion,
)


class BaseAPITestCase(TestCase):
    def setUp(self):
        self.client = APIClient()
        self.video = Video.objects.create(id="vid_123")


# =========================
# SaveExpertAnnotationAPIView
# =========================

class SaveExpertAnnotationAPIViewTestCase(BaseAPITestCase):

    def setUp(self):
        super().setUp()
        self.url = reverse('expert-annotations')

    def test_create_annotation_success(self):
        response = self.client.post(
            self.url,
            {
                "mode": "create",
                "video_id": self.video.id,
                "start": 0,
                "end": 10,
                "question_type": "mcq",
                "question": "What is this?",
                "answer": "Test answer",
            },
            format="json",
        )

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertTrue(response.data["success"])
        self.assertEqual(ExpertAnnotation.objects.count(), 1)

    def test_create_annotation_skip(self):
        response = self.client.post(
            self.url,
            {
                "mode": "create",
                "video_id": self.video.id,
                "start": 0,
                "end": 10,
                "skip": True,
            },
            format="json",
        )

        self.assertEqual(response.status_code, status.HTTP_200_OK)

        obj = ExpertAnnotation.objects.first()
        self.assertTrue(obj.skipped)
        self.assertEqual(obj.question_type, "skip")

    def test_missing_video_id(self):
        response = self.client.post(
            self.url,
            {"start": 0, "end": 10},
            format="json",
        )

        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)


# =========================
# ExpertQuestionsByVideoAPIView
# =========================

class ExpertQuestionsByVideoAPIViewTestCase(BaseAPITestCase):

    def setUp(self):
        super().setUp()
        self.url = reverse('expert-questions-by-video', args=[self.video.id])

    def test_get_questions(self):
        ExpertQuestion.objects.create(
            video=self.video,
            segment_start=0,
            segment_end=5,
            timestamp=1,
            skipped=False,
            question_type="mcq",
            question="Q1",
            answer="A1",
        )

        response = self.client.get(self.url)

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(len(response.data["questions"]), 1)
        self.assertEqual(response.data["questions"][0]["question"], "Q1")


# =========================
# SaveExpertQuestionsAPIView
# =========================

class SaveExpertQuestionsAPIViewTestCase(BaseAPITestCase):

    def setUp(self):
        super().setUp()
        self.url = reverse('expert-questions-save')

    def test_save_bulk_questions(self):
        response = self.client.post(
            self.url,
            {
                "video_id": self.video.id,
                "questions": [
                    {
                        "segmentStart": 0,
                        "segmentEnd": 5,
                        "timestamp": 1,
                        "questionType": "mcq",
                        "question": "Q1",
                        "answer": "A1",
                    },
                    {
                        "segmentStart": 5,
                        "segmentEnd": 10,
                        "timestamp": 6,
                        "questionType": "mcq",
                        "question": "Q2",
                        "answer": "A2",
                    },
                ],
            },
            format="json",
        )

        self.assertEqual(response.status_code, status.HTTP_201_CREATED)
        self.assertEqual(response.data["saved"], 2)
        self.assertEqual(ExpertQuestion.objects.count(), 2)

    def test_save_single_question(self):
        response = self.client.post(
            self.url,
            {
                "video_id": self.video.id,
                "segmentStart": 0,
                "segmentEnd": 5,
                "timestamp": 1,
                "questionType": "mcq",
                "question": "Q1",
                "answer": "A1",
            },
            format="json",
        )

        self.assertEqual(response.status_code, status.HTTP_201_CREATED)
        self.assertEqual(ExpertQuestion.objects.count(), 1)


# =========================
# SaveFinalQuestionsAPIView
# =========================

class SaveFinalQuestionsAPIViewTestCase(BaseAPITestCase):

    def setUp(self):
        super().setUp()
        self.url = reverse('save-final-questions')

    def test_save_final_questions(self):
        response = self.client.post(
            self.url,
            {
                "video_id": self.video.id,
                "questions": [
                    {
                        "start": 0,
                        "end": 10,
                        "result": {
                            "questions": {
                                "mcq": {
                                    "q": "What?",
                                    "a": "Answer",
                                    "rank": 1,
                                    "followupForCorrectAnswer": {
                                        "q": "Why?",
                                        "a": "Because",
                                        "rank": 1,
                                    },
                                    "followupForIncorrectAnswer": {
                                        "q": "Try again?",
                                        "a": "Think again",
                                        "rank": 1,
                                    },
                                }
                            }
                        },
                    }
                ],
            },
            format="json",
        )

        self.assertEqual(response.status_code, status.HTTP_201_CREATED)
        self.assertEqual(FinalQuestionSet.objects.count(), 1)
        self.assertEqual(FinalSegment.objects.count(), 1)
        self.assertEqual(FinalAIQuestion.objects.count(), 1)


# =========================
# FinalQuestionsForKidsAPIView
# =========================

class FinalQuestionsForKidsAPIViewTestCase(BaseAPITestCase):

    def setUp(self):
        super().setUp()
        self.url = reverse('final-questions-kids', args=[self.video.id])

    def test_from_final_set(self):
        final_set = FinalQuestionSet.objects.create(video=self.video, payload={})

        seg = FinalSegment.objects.create(
            final_set=final_set,
            start_seconds=0,
            end_seconds=10,
        )

        FinalAIQuestion.objects.create(
            final_segment=seg,
            question="Best Q",
            answer="Best A",
            llm_ranking=1,
            trashed=False,
        )

        response = self.client.get(self.url)

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(response.data["segments"][0]["question"], "Best Q")

    def test_fallback_to_submitted(self):
        SubmittedQuestionSet.objects.create(
            video_id=self.video.id,
            payload={
                "segments": [
                    {
                        "start": 0,
                        "end": 10,
                        "result": {
                            "questions": {
                                "mcq": {
                                    "q": "Fallback Q",
                                    "a": "Fallback A",
                                    "rank": 1,
                                }
                            }
                        },
                    }
                ]
            },
        )

        response = self.client.get(self.url)

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(response.data["segments"][0]["question"], "Fallback Q")

    def test_not_found(self):
        url = reverse('final-questions-kids', args=["unknown"])

        response = self.client.get(url)

        self.assertEqual(response.status_code, status.HTTP_404_NOT_FOUND)
        self.assertFalse(response.data["success"])