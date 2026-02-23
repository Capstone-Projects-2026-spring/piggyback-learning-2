from rest_framework.parsers import FormParser, MultiPartParser
from rest_framework.response import Response
from rest_framework.views import APIView


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

    def post(self, request):
        user_type = (request.data.get('user_type') or '').strip()
        password = (request.data.get('password') or '').strip()

        valid_passwords = {
            'admin': 'admin123',
            'expert': 'expert123',
        }

        if user_type in valid_passwords and password == valid_passwords[user_type]:
            if user_type == 'admin':
                return Response({'success': True, 'redirect': '/admin'})
            if user_type == 'expert':
                return Response({'success': True, 'redirect': '/expert-preview'})

        return Response({'success': False, 'message': 'Invalid password'})
