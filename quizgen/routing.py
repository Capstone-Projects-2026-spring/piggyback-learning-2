from django.urls import re_path

from .consumers import QuestionsConsumer

websocket_urlpatterns = [
    re_path(r'^ws/questions/(?P<video_id>[^/]+)/?$', QuestionsConsumer.as_asgi()),
]
