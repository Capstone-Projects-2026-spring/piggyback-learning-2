from rest_framework import status
from rest_framework.parsers import FormParser, MultiPartParser
from rest_framework.response import Response
from rest_framework.views import APIView

from .serializers import (
    VerifyPasswordFailureSerializer,
    VerifyPasswordRequestSerializer,
    VerifyPasswordSuccessSerializer,
)


class VerifyPasswordAPIView(APIView):
    """
    POST /api/verify-password
    FastAPI expects Form(...) fields: user_type, password
    Returns:
      {"success": True, "redirect": "/admin"} or "/expert-preview"
      {"success": False, "message": "Invalid password"}
    """

    authentication_classes = []
    permission_classes = []
    parser_classes = [FormParser, MultiPartParser]
    serializer_class = VerifyPasswordRequestSerializer

    def post(self, request):
        user_type = (request.data.get('user_type') or '').strip()
        password = (request.data.get('password') or '').strip()

        valid_passwords = {
            'admin': 'admin123',
            'expert': 'expert123',
        }

        if password == valid_passwords.get(user_type):
            redirect = '/admin' if user_type == 'admin' else '/expert-preview'
            return Response(
                VerifyPasswordSuccessSerializer(
                    {'success': True, 'redirect': redirect}
                ).data
            )

        return Response(
            VerifyPasswordFailureSerializer(
                {'success': False, 'message': 'Invalid password'}
            ).data,
            status=status.HTTP_400_BAD_REQUEST,
        )
