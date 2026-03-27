from django.urls import path

from .views import (
    AdminVideosAPIView,
    DownloadAPIView,
    ExtractFramesAPIView,
    SubmitQuestionsAPIView,
    VideoQuestionsAPIView,
)

urlpatterns = [
    path('admin/videos', AdminVideosAPIView.as_view(), name='admin-videos'),
    path('submit-questions', SubmitQuestionsAPIView.as_view(), name='submit-questions'),
    path('download', DownloadAPIView.as_view(), name='admin-download'),
    path(
        'questions/<str:video_id>',
        VideoQuestionsAPIView.as_view(),
        name='video-questions',
    ),
    path(
        'frames/<str:video_id>', ExtractFramesAPIView.as_view(), name='extract-frames'
    ),
]
