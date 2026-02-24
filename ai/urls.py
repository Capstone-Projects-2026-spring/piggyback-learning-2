from django.urls import path

from .views import CheckAnswerAPIView, ConfigAPIView, TranscribeAPIView, TTSAPIView

urlpatterns = [
    path('check_answer', CheckAnswerAPIView.as_view(), name='check-answer'),
    path('transcribe', TranscribeAPIView.as_view(), name='transcribe'),
    path('config', ConfigAPIView.as_view(), name='config'),
    path('tts', TTSAPIView.as_view(), name='tts'),
]
