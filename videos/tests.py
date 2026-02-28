from django.test import TestCase
from django.urls import reverse


# Create your tests here.

# Very basic, verifies urls are active / return 200 on a GET
class urlsTests(TestCase):
    def test_kids_videos_endpoint(self):
        url = reverse('kids-videos')
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)

    def test_video_list_endpoint(self):
        url = reverse('video-list')
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)