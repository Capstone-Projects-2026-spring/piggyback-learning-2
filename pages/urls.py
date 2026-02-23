from django.urls import path

from . import views

urlpatterns = [
    path('', views.home, name='home'),
    path('home', views.home, name='home-alt'),
    path('children', views.children, name='children'),
    path('admin', views.admin_ui, name='admin-ui'),  # keep FastAPI URL
    path('expert-preview', views.expert_preview, name='expert-preview'),
]
