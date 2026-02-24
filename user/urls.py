from django.urls import path

from .views import VerifyPasswordAPIView

urlpatterns = [
    path('verify-password', VerifyPasswordAPIView.as_view(), name='verify-password'),
]
