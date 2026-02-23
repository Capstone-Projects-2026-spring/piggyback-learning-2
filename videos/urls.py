from django.urls import path

from .views import KidsVideosAPIView

urlpatterns = [
    path('kids_videos', KidsVideosAPIView.as_view(), name='kids-videos'),
]
