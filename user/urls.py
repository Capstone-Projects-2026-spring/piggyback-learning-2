from django.urls import path

from .views import (
    AdminChildDeactivateAPIView,
    AdminChildDetailAPIView,
    AdminChildrenListCreateAPIView,
    AdminChildUnlinkAPIView,
    AdminExpertDeactivateAPIView,
    AdminExpertDetailAPIView,
    AdminExpertListCreateAPIView,
    ExpertLoginAPIView,
    ExpertLogoutAPIView,
    LearnerChildrenByExpertAPIView,
    LearnerVideosByChildAPIView,
    VerifyPasswordAPIView,
)

urlpatterns = [
    # existing
    path('verify-password', VerifyPasswordAPIView.as_view(), name='verify-password'),

    # expert auth
    path('expert/login', ExpertLoginAPIView.as_view(), name='expert-login'),
    path('expert/logout', ExpertLogoutAPIView.as_view(), name='expert-logout'),

    # learner
    path('learners/experts/<str:expert_id>/children', LearnerChildrenByExpertAPIView.as_view(), name='learner-children-by-expert'),
    path('learners/children/<str:child_id>/videos', LearnerVideosByChildAPIView.as_view(), name='learner-videos-by-child'),

    # admin experts
    path('admin/experts', AdminExpertListCreateAPIView.as_view(), name='admin-experts'),
    path('admin/experts/<str:expert_id>', AdminExpertDetailAPIView.as_view(), name='admin-expert-detail'),
    path('admin/experts/<str:expert_id>/deactivate', AdminExpertDeactivateAPIView.as_view(), name='admin-expert-deactivate'),

    # admin children
    path('admin/children', AdminChildrenListCreateAPIView.as_view(), name='admin-children'),
    path('admin/children/<str:child_id>', AdminChildDetailAPIView.as_view(), name='admin-child-detail'),
    path('admin/children/<str:child_id>/unlink', AdminChildUnlinkAPIView.as_view(), name='admin-child-unlink'),
    path('admin/children/<str:child_id>/deactivate', AdminChildDeactivateAPIView.as_view(), name='admin-child-deactivate'),
]
