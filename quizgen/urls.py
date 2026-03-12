from django.urls import path

from .views import (
    AdminVideosAPIView,
    DownloadAPIView,
    ExtractFramesAPIView,
    GetQuizScoresAPIView,
    SaveQuizScoreAPIView,
    SubmitQuestionsAPIView,
    VideoQuestionsAPIView,
)

urlpatterns = [
    # existing
    path('admin/videos', AdminVideosAPIView.as_view(), name='admin-videos'),
    path('submit-questions', SubmitQuestionsAPIView.as_view(), name='submit-questions'),
    path('download', DownloadAPIView.as_view(), name='admin-download'),
    path('questions/<str:video_id>', VideoQuestionsAPIView.as_view(), name='video-questions'),
    path('frames/<str:video_id>', ExtractFramesAPIView.as_view(), name='extract-frames'),

    # quiz scores
    path('save-quiz-score', SaveQuizScoreAPIView.as_view(), name='save-quiz-score'),
    path('get-quiz-scores/<str:child_id>', GetQuizScoresAPIView.as_view(), name='get-quiz-scores'),
]
