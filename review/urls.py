from django.urls import path

from .views import (
    ExpertQuestionsByVideoAPIView,
    FinalQuestionsForKidsAPIView,
    SaveExpertAnnotationAPIView,
    SaveExpertQuestionsAPIView,
    SaveFinalQuestionsAPIView,
)

urlpatterns = [
    path(
        'expert-annotations',
        SaveExpertAnnotationAPIView.as_view(),
        name='expert-annotations',
    ),
    path(
        'expert-questions/<str:video_id>',
        ExpertQuestionsByVideoAPIView.as_view(),
        name='expert-questions-by-video',
    ),
    path(
        'expert-questions',
        SaveExpertQuestionsAPIView.as_view(),
        name='expert-questions-save',
    ),
    path(
        'save-final-questions',
        SaveFinalQuestionsAPIView.as_view(),
        name='save-final-questions',
    ),
    path(
        'final-questions/<str:video_id>',
        FinalQuestionsForKidsAPIView.as_view(),
        name='final-questions-kids',
    ),
]
