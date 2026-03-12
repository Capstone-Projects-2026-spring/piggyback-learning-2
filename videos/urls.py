from django.urls import path

from .views import (
    AdminVideoAssignmentsAPIView,
    ExpertClaimVideoAPIView,
    ExpertVideosAPIView,
    KidsVideosAPIView,
    VideoListAPIView,
)

urlpatterns = [
    # existing
    path('kids_videos', KidsVideosAPIView.as_view(), name='kids-videos'),
    path('videos-list', VideoListAPIView.as_view(), name='video-list'),

    # expert videos
    path('expert/videos', ExpertVideosAPIView.as_view(), name='expert-videos'),
    path('expert/videos/<str:video_id>/claim', ExpertClaimVideoAPIView.as_view(), name='expert-claim-video'),

    # admin video assignments
    path('admin/videos/assignments', AdminVideoAssignmentsAPIView.as_view(), name='admin-video-assignments'),
]
