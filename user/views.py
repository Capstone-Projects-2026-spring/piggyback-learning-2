from django.contrib.auth.hashers import check_password, make_password
from rest_framework import status
from rest_framework.parsers import FormParser, MultiPartParser
from rest_framework.response import Response
from rest_framework.views import APIView

from .models import ALLOWED_CHILD_ICON_KEYS, Child, Expert
from .serializers import (
    VerifyPasswordFailureSerializer,
    VerifyPasswordRequestSerializer,
    VerifyPasswordSuccessSerializer,
)


class VerifyPasswordAPIView(APIView):
    """
    POST /api/verify-password
    FastAPI expects Form(...) fields: user_type, password
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


# ── Expert Auth ────────────────────────────────────────────────────────────────

class ExpertLoginAPIView(APIView):
    """POST /api/expert/login"""

    authentication_classes = []
    permission_classes = []

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        expert_id = str(payload.get('expert_id') or '').strip().lower()
        password = str(payload.get('password') or '')

        if not expert_id or not password:
            return Response(
                {'success': False, 'message': 'expert_id and password are required'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        try:
            expert = Expert.objects.get(expert_id=expert_id, is_active=True)
        except Expert.DoesNotExist:
            return Response(
                {'success': False, 'message': 'Invalid expert ID or password'},
                status=status.HTTP_401_UNAUTHORIZED,
            )

        if not check_password(password, expert.password_hash):
            return Response(
                {'success': False, 'message': 'Invalid expert ID or password'},
                status=status.HTTP_401_UNAUTHORIZED,
            )

        request.session['role'] = 'expert'
        request.session['expert_id'] = expert.expert_id
        request.session['display_name'] = expert.display_name or expert.expert_id

        return Response({
            'success': True,
            'redirect': '/expert-preview',
            'expert': expert.to_dict(),
        })


class ExpertLogoutAPIView(APIView):
    """POST /api/expert/logout"""

    authentication_classes = []
    permission_classes = []

    def post(self, request):
        request.session.flush()
        return Response({'success': True})


# ── Learner endpoints ──────────────────────────────────────────────────────────

class LearnerChildrenByExpertAPIView(APIView):
    """GET /api/learners/experts/{expert_id}/children"""

    authentication_classes = []
    permission_classes = []

    def get(self, request, expert_id: str):
        expert_id = (expert_id or '').strip().lower()
        if not expert_id:
            return Response(
                {'success': False, 'message': 'expert_id is required', 'children': []},
                status=status.HTTP_400_BAD_REQUEST,
            )

        try:
            expert = Expert.objects.get(expert_id=expert_id, is_active=True)
        except Expert.DoesNotExist:
            return Response(
                {'success': False, 'message': 'Expert not found', 'children': []},
                status=status.HTTP_404_NOT_FOUND,
            )

        children = list(
            expert.children.filter(is_active=True).values(
                'child_id', 'first_name', 'last_name', 'icon_key', 'is_active', 'expert_id'
            )
        )

        return Response({
            'success': True,
            'expert': expert.to_dict(),
            'children': children,
            'count': len(children),
        })


class LearnerVideosByChildAPIView(APIView):
    """GET /api/learners/children/{child_id}/videos"""

    authentication_classes = []
    permission_classes = []

    def get(self, request, child_id: str):
        try:
            child = Child.objects.get(child_id=child_id, is_active=True)
        except Child.DoesNotExist:
            return Response(
                {'success': False, 'message': 'Child not found', 'videos': []},
                status=status.HTTP_404_NOT_FOUND,
            )

        if not child.expert_id:
            return Response({
                'success': True,
                'child': child.to_dict(),
                'videos': [],
                'count': 0,
                'message': 'Child is not linked to an expert',
            })

        # Import here to avoid circular imports
        from videos.models import VideoAssignment

        assigned_video_ids = set(
            VideoAssignment.objects.filter(
                expert_id=child.expert_id
            ).values_list('video_id', flat=True)
        )

        if not assigned_video_ids:
            return Response({
                'success': True,
                'child': child.to_dict(),
                'videos': [],
                'count': 0,
            })

        from videos.models import Video
        from videos.views import _format_mmss

        videos = []
        for v in Video.objects.filter(id__in=assigned_video_ids):
            videos.append({
                'video_id': v.id,
                'title': v.title or v.id,
                'duration': _format_mmss(v.duration_seconds),
                'local_path': v.local_video_path or '',
                'thumbnail': v.thumbnail_url or '',
            })

        return Response({
            'success': True,
            'child': child.to_dict(),
            'videos': videos,
            'count': len(videos),
        })


# ── Admin: Expert management ────────────────────────────────────────────────────

class AdminExpertListCreateAPIView(APIView):
    """GET/POST /api/admin/experts"""

    authentication_classes = []
    permission_classes = []

    def get(self, request):
        experts = [e.to_dict() for e in Expert.objects.all().order_by('expert_id')]
        return Response({'success': True, 'experts': experts})

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        expert_id = str(payload.get('expert_id') or '').strip().lower()
        display_name = str(payload.get('display_name') or '').strip()
        password = str(payload.get('password') or '')

        if not expert_id:
            return Response(
                {'detail': 'expert_id is required'}, status=status.HTTP_400_BAD_REQUEST
            )
        if not password:
            return Response(
                {'detail': 'password is required'}, status=status.HTTP_400_BAD_REQUEST
            )
        if Expert.objects.filter(expert_id=expert_id).exists():
            return Response(
                {'detail': 'expert_id already exists'}, status=status.HTTP_409_CONFLICT
            )

        expert = Expert(expert_id=expert_id, display_name=display_name)
        expert.set_password(password)
        expert.save()

        return Response(
            {'success': True, 'expert': expert.to_dict()},
            status=status.HTTP_201_CREATED,
        )


class AdminExpertDetailAPIView(APIView):
    """PUT/DELETE /api/admin/experts/{expert_id}"""

    authentication_classes = []
    permission_classes = []

    def put(self, request, expert_id: str):
        try:
            expert = Expert.objects.get(expert_id=expert_id)
        except Expert.DoesNotExist:
            return Response({'detail': 'expert not found'}, status=status.HTTP_404_NOT_FOUND)

        payload = request.data if isinstance(request.data, dict) else {}
        display_name = payload.get('display_name')
        password = payload.get('password')
        is_active = payload.get('is_active')

        if display_name is not None:
            expert.display_name = str(display_name)
        if password is not None:
            expert.set_password(str(password))
        if is_active is not None:
            if not isinstance(is_active, bool):
                return Response(
                    {'detail': 'is_active must be true or false'},
                    status=status.HTTP_400_BAD_REQUEST,
                )
            expert.is_active = is_active

        expert.save()
        return Response({'success': True, 'expert': expert.to_dict()})

    def delete(self, request, expert_id: str):
        try:
            expert = Expert.objects.get(expert_id=expert_id)
        except Expert.DoesNotExist:
            return Response({'detail': 'expert not found'}, status=status.HTTP_404_NOT_FOUND)

        if expert.children.exists():
            return Response(
                {'detail': 'cannot delete expert while children are linked; reassign or deactivate children first'},
                status=status.HTTP_409_CONFLICT,
            )

        expert.delete()
        return Response({'success': True})


class AdminExpertDeactivateAPIView(APIView):
    """POST /api/admin/experts/{expert_id}/deactivate"""

    authentication_classes = []
    permission_classes = []

    def post(self, request, expert_id: str):
        try:
            expert = Expert.objects.get(expert_id=expert_id)
        except Expert.DoesNotExist:
            return Response({'detail': 'expert not found'}, status=status.HTTP_404_NOT_FOUND)

        expert.is_active = False
        expert.save()
        return Response({'success': True, 'expert': expert.to_dict()})


# ── Admin: Children management ─────────────────────────────────────────────────

class AdminChildrenListCreateAPIView(APIView):
    """GET/POST /api/admin/children"""

    authentication_classes = []
    permission_classes = []

    def get(self, request):
        expert_id = request.query_params.get('expert_id')
        include_inactive = request.query_params.get('include_inactive', 'false').lower() in {'1', 'true', 'yes'}

        qs = Child.objects.all()
        if expert_id:
            qs = qs.filter(expert_id=expert_id)
        if not include_inactive:
            qs = qs.filter(is_active=True)

        children = [c.to_dict() for c in qs.order_by('first_name', 'last_name')]
        experts = [e.to_dict() for e in Expert.objects.all().order_by('expert_id')]

        return Response({
            'success': True,
            'children': children,
            'experts': experts,
            'icon_keys': list(ALLOWED_CHILD_ICON_KEYS),
            'count': len(children),
        })

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        expert_id = str(payload.get('expert_id') or '').strip().lower()
        first_name = str(payload.get('first_name') or '').strip()
        last_name = str(payload.get('last_name') or '').strip()
        icon_key = str(payload.get('icon_key') or '').strip().lower()

        if not first_name:
            return Response({'detail': 'first_name is required'}, status=status.HTTP_400_BAD_REQUEST)
        if icon_key and icon_key not in ALLOWED_CHILD_ICON_KEYS:
            return Response({'detail': f'invalid icon_key: {icon_key}'}, status=status.HTTP_400_BAD_REQUEST)

        expert = None
        if expert_id:
            try:
                expert = Expert.objects.get(expert_id=expert_id)
            except Expert.DoesNotExist:
                return Response({'detail': 'expert not found'}, status=status.HTTP_404_NOT_FOUND)

        if Child.objects.filter(expert=expert, first_name=first_name, last_name=last_name).exists():
            return Response({'detail': 'duplicate child profile for this expert'}, status=status.HTTP_409_CONFLICT)

        child = Child.objects.create(
            expert=expert,
            first_name=first_name,
            last_name=last_name,
            icon_key=icon_key,
        )
        return Response({'success': True, 'child': child.to_dict()}, status=status.HTTP_201_CREATED)


class AdminChildDetailAPIView(APIView):
    """PUT /api/admin/children/{child_id}"""

    authentication_classes = []
    permission_classes = []

    def put(self, request, child_id: str):
        try:
            child = Child.objects.get(child_id=child_id)
        except Child.DoesNotExist:
            return Response({'detail': 'child not found'}, status=status.HTTP_404_NOT_FOUND)

        payload = request.data if isinstance(request.data, dict) else {}
        expert_id = payload.get('expert_id')
        first_name = payload.get('first_name')
        last_name = payload.get('last_name')
        icon_key = payload.get('icon_key')
        is_active = payload.get('is_active')

        if expert_id is not None:
            expert_id_str = str(expert_id).strip().lower()
            if expert_id_str == '':
                child.expert = None
            else:
                try:
                    child.expert = Expert.objects.get(expert_id=expert_id_str)
                except Expert.DoesNotExist:
                    return Response({'detail': 'expert not found'}, status=status.HTTP_404_NOT_FOUND)

        if first_name is not None:
            child.first_name = str(first_name).strip()
        if last_name is not None:
            child.last_name = str(last_name).strip()
        if icon_key is not None:
            icon_key_str = str(icon_key).strip().lower()
            if icon_key_str and icon_key_str not in ALLOWED_CHILD_ICON_KEYS:
                return Response({'detail': f'invalid icon_key: {icon_key_str}'}, status=status.HTTP_400_BAD_REQUEST)
            child.icon_key = icon_key_str
        if is_active is not None:
            if not isinstance(is_active, bool):
                return Response({'detail': 'is_active must be true or false'}, status=status.HTTP_400_BAD_REQUEST)
            child.is_active = is_active

        child.save()
        return Response({'success': True, 'child': child.to_dict()})


class AdminChildUnlinkAPIView(APIView):
    """POST /api/admin/children/{child_id}/unlink"""

    authentication_classes = []
    permission_classes = []

    def post(self, request, child_id: str):
        try:
            child = Child.objects.get(child_id=child_id)
        except Child.DoesNotExist:
            return Response({'detail': 'child not found'}, status=status.HTTP_404_NOT_FOUND)

        child.expert = None
        child.save()
        return Response({'success': True, 'child': child.to_dict()})


class AdminChildDeactivateAPIView(APIView):
    """POST /api/admin/children/{child_id}/deactivate"""

    authentication_classes = []
    permission_classes = []

    def post(self, request, child_id: str):
        try:
            child = Child.objects.get(child_id=child_id)
        except Child.DoesNotExist:
            return Response({'detail': 'child not found'}, status=status.HTTP_404_NOT_FOUND)

        child.is_active = False
        child.save()
        return Response({'success': True, 'child': child.to_dict()})
