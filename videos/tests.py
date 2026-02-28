from django.test import TestCase
from django.urls import reverse
from rest_framework import status
from .models import Video


# Create your tests here.

class VideosUnitTests(TestCase):
    # videos/urls.py tests ####################################################################
    # verifies urls are active / return 200 on a GET
    def test_kids_videos_endpoint(self):
        url = reverse('kids-videos')
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)

    def test_video_list_endpoint(self):
        url = reverse('video-list')
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)

    # videos/views.py tests ######################################################################
    def setUp(self):
        """
        Create test video objects.
        Adjust field names if your Video model differs.
        """

        self.video1 = Video.objects.create(
            id="abc123",
            title="Test Video 1",
            duration_seconds=125,
            local_video_path="/videos/test1.mp4",
            thumbnail_url="/thumbs/test1.jpg",
        )

        self.video2 = Video.objects.create(
            id="xyz789",
            title="Test Video 2",
            duration_seconds=65,
            local_video_path="/videos/test2.mp4",
            thumbnail_url="/thumbs/test2.jpg",
        )

    def test_kids_videos_success(self):
        print("\nRunning test_kids_videos_success")

        url = reverse("kids-videos")
        response = self.client.get(url)

        print("Response JSON:", response.json())

        self.assertEqual(response.status_code, status.HTTP_200_OK)

        data = response.json()

        self.assertTrue(data["success"])
        self.assertEqual(data["count"], 2)
        self.assertEqual(len(data["videos"]), 2)

    def test_kids_videos_duration_format(self):
        print("\nRunning test_kids_videos_duration_format")

        url = reverse("kids-videos")
        response = self.client.get(url)
        data = response.json()

        first_video = data["videos"][0]

        # 125 seconds = 02:05
        self.assertEqual(first_video["duration"], "02:05")

    def test_kids_videos_empty_db(self):
        print("\nRunning test_kids_videos_empty_db")

        Video.objects.all().delete()

        url = reverse("kids-videos")
        response = self.client.get(url)
        data = response.json()

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(data["count"], 0)
        self.assertEqual(data["videos"], [])







