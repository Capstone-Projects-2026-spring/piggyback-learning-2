from django.urls import path

from .views import KidsVideosAPIView, VideoListAPIView

urlpatterns = [
    path('kids_videos', KidsVideosAPIView.as_view(), name='kids-videos'),
    path('videos-list', VideoListAPIView.as_view(), name='video-list'),
]
