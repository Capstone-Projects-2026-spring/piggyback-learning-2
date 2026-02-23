from django.contrib import admin
from django.urls import include, path

from .swagger import schema_view

urlpatterns = [
    path('admin/', admin.site.urls),
    path('swagger/', schema_view.with_ui('swagger', cache_timeout=0)),
    path('redoc/', schema_view.with_ui('redoc', cache_timeout=0)),
    path('api/', include('ai.urls')),
    path('api/', include('videos.urls')),
    path('api/', include('quizgen.urls')),
    path('api/', include('review.urls')),
]
