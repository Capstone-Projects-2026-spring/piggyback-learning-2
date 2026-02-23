from django.conf import settings
from django.conf.urls.static import static
from django.contrib import admin
from django.urls import include, path
from django.views.static import serve as static_serve

from .swagger import schema_view

urlpatterns = [
    path('django-admin/', admin.site.urls),
    path('swagger/', schema_view.with_ui('swagger', cache_timeout=0)),
    path('redoc/', schema_view.with_ui('redoc', cache_timeout=0)),
    path('', include('pages.urls')),
    path('api/', include('ai.urls')),
    path('api/', include('videos.urls')),
    path('api/', include('quizgen.urls')),
    path('api/', include('review.urls')),
    path('api/', include('user.urls')),
]
# ---- DEV-ONLY static serving to match FastAPI mounts ----
# /downloads/... -> BASE_DIR/downloads
urlpatterns += [
    path(
        'downloads/<path:path>', static_serve, {'document_root': settings.DOWNLOADS_DIR}
    ),
]

# /assets/... -> BASE_DIR/public/assets
urlpatterns += [
    path(
        'assets/<path:path>',
        static_serve,
        {'document_root': settings.PUBLIC_ASSETS_DIR},
    ),
]

# /static/... -> your static folder(s) (in dev)
urlpatterns += static(settings.STATIC_URL, document_root=settings.BASE_DIR / 'static')
