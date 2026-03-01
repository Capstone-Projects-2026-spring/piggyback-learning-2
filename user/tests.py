from django.test import TestCase
from django.urls import reverse
from rest_framework.test import APIClient
from rest_framework import status


class VerifyPasswordAPIViewTestCase(TestCase):

    def setUp(self):
        self.client = APIClient()
        self.url = reverse('verify-password')

    """Checks if verify_password returns success for correct admin password."""
    def test_verify_password_success_admin(self):
        response = self.client.post(
            self.url,
            {'user_type': 'admin', 'password': 'admin123'},
            format='multipart'
        )
        
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(response.data['success'], True)
        self.assertEqual(response.data['redirect'], '/admin')

    """Checks if verify_password returns success for correct expert password."""
    def test_verify_password_success_expert(self):
        response = self.client.post(
            self.url,
            {'user_type': 'expert', 'password': 'expert123'},
            format='multipart'
        )
        
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(response.data['success'], True)
        self.assertEqual(response.data['redirect'], '/expert-preview')

    """Checks if verify_password returns failure for incorrect password."""
    def test_verify_password_failure_invalid_password(self):
        response = self.client.post(
            self.url,
            {'user_type': 'admin', 'password': 'wrongpassword'},
            format='multipart'
        )
        
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertEqual(response.data['success'], False)
        self.assertEqual(response.data['message'], 'Invalid password')

    """Checks if verify_password returns failure for non-existent user type."""
    def test_verify_password_failure_invalid_user_type(self):
        response = self.client.post(
            self.url,
            {'user_type': 'invalid', 'password': 'admin123'},
            format='multipart'
        )
        
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertEqual(response.data['success'], False)
        self.assertEqual(response.data['message'], 'Invalid password')

