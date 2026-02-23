from django.urls import path

from .views import (
    AdminVideosAPIView,
    DownloadAPIView,
    ExtractFramesAPIView,
    SubmitQuestionsAPIView,
)

urlpatterns = [
    path('admin/videos', AdminVideosAPIView.as_view(), name='admin-videos'),
    path('submit-questions', SubmitQuestionsAPIView.as_view(), name='submit-questions'),
    path('download', DownloadAPIView.as_view(), name='admin-download'),
    path(
        'frames/<str:video_id>', ExtractFramesAPIView.as_view(), name='extract-frames'
    ),
]
