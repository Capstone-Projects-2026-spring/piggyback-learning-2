from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView

from .models import Video, VideoAssignment
from .serializers import VideoSerializer


def _format_mmss(sec: int | None) -> str:
    if not sec or sec < 0:
        return '00:00'
    m, s = divmod(int(sec), 60)
    return f'{m:02d}:{s:02d}'


class KidsVideosAPIView(APIView):
    """GET /api/kids_videos"""

    authentication_classes = []
    permission_classes = []

    def get(self, request):
        qs = Video.objects.all().order_by('-created_at')
        data = VideoSerializer(qs, many=True).data

        videos = []
        for row in data:
            dur = row.get('duration_seconds')
            videos.append(
                {
                    'video_id': row['video_id'],
                    'title': row.get('title') or row['video_id'],
                    'duration': _format_mmss(dur),
                    'local_path': row.get('local_path') or '',
                    'thumbnail': row.get('thumbnail') or '',
                }
            )

        return Response({'success': True, 'count': len(videos), 'videos': videos})


class VideoListAPIView(APIView):
    """GET /api/videos-list"""

    authentication_classes = []
    permission_classes = []

    def get(self, request):
        qs = Video.objects.all().order_by('-created_at')
        videos = []
        for v in qs:
            videos.append(
                {
                    'id': v.id,
                    'title': v.title or '',
                    'thumbnail': v.thumbnail_url or '',
                    'duration': v.duration_seconds,
                    'local_path': v.local_video_path or '',
                }
            )

        return Response({'videos': videos, 'success': True}, status=status.HTTP_200_OK)


# ── Expert video endpoints ─────────────────────────────────────────────────────

def _require_expert_session(request):
    """Returns (expert_id, display_name) or None if not authenticated."""
    if request.session.get('role') != 'expert':
        return None
    expert_id = request.session.get('expert_id')
    if not expert_id:
        return None
    return {
        'expert_id': str(expert_id),
        'display_name': str(request.session.get('display_name') or expert_id),
    }


class ExpertVideosAPIView(APIView):
    """GET /api/expert/videos — returns videos assigned to the logged-in expert"""

    authentication_classes = []
    permission_classes = []

    def get(self, request):
        expert_identity = _require_expert_session(request)
        if not expert_identity:
            return Response(
                {'success': False, 'message': 'Expert login required', 'videos': []},
                status=status.HTTP_403_FORBIDDEN,
            )

        expert_id = expert_identity['expert_id']
        assigned = VideoAssignment.objects.filter(
            expert_id=expert_id
        ).select_related('video')

        videos = []
        for assignment in assigned:
            v = assignment.video
            all_assigned = VideoAssignment.objects.filter(video_id=v.id)
            videos.append(
                {
                    'id': v.id,
                    'title': v.title or v.id,
                    'thumbnail': v.thumbnail_url or '',
                    'duration': v.duration_seconds,
                    'videoUrl': v.local_video_path or '',
                    'assigned_to_me': True,
                    'assigned_expert_count': all_assigned.count(),
                }
            )

        return Response({'success': True, 'videos': videos})


class ExpertClaimVideoAPIView(APIView):
    """POST /api/expert/videos/{video_id}/claim"""

    authentication_classes = []
    permission_classes = []

    def post(self, request, video_id: str):
        expert_identity = _require_expert_session(request)
        if not expert_identity:
            return Response(
                {'success': False, 'message': 'Expert login required'},
                status=status.HTTP_403_FORBIDDEN,
            )

        try:
            assignment = VideoAssignment.objects.get(
                video_id=video_id, expert_id=expert_identity['expert_id']
            )
        except VideoAssignment.DoesNotExist:
            return Response(
                {'success': False, 'message': 'Video is not assigned to this expert'},
                status=status.HTTP_403_FORBIDDEN,
            )

        assignment.claimed = True
        assignment.save()
        return Response({'success': True})


# ── Admin: video assignments ───────────────────────────────────────────────────

class AdminVideoAssignmentsAPIView(APIView):
    """GET/POST /api/admin/videos/assignments"""

    authentication_classes = []
    permission_classes = []

    def get(self, request):
        from user.models import Expert

        videos = Video.objects.all().order_by('id')
        experts = [e.to_dict() for e in Expert.objects.all().order_by('expert_id')]

        rows = []
        for v in videos:
            assigned = VideoAssignment.objects.filter(video=v).select_related('expert')
            rows.append(
                {
                    'video_id': v.id,
                    'title': v.title or v.id,
                    'duration_seconds': v.duration_seconds,
                    'assigned_experts': [
                        {
                            'expert_id': a.expert_id,
                            'display_name': a.expert.display_name or a.expert_id,
                        }
                        for a in assigned
                    ],
                }
            )

        return Response({'success': True, 'experts': experts, 'assignments': rows})

    def post(self, request):
        from user.models import Expert

        payload = request.data if isinstance(request.data, dict) else {}
        video_id = str(payload.get('video_id') or '').strip()
        expert_id = str(payload.get('expert_id') or '').strip()
        op = str(payload.get('op') or '').strip()

        if not video_id:
            return Response({'detail': 'video_id is required'}, status=status.HTTP_400_BAD_REQUEST)
        if not expert_id:
            return Response({'detail': 'expert_id is required'}, status=status.HTTP_400_BAD_REQUEST)
        if op not in {'add', 'remove'}:
            return Response({'detail': "op must be 'add' or 'remove'"}, status=status.HTTP_400_BAD_REQUEST)

        try:
            video = Video.objects.get(id=video_id)
        except Video.DoesNotExist:
            return Response({'detail': 'video not found'}, status=status.HTTP_404_NOT_FOUND)

        try:
            expert = Expert.objects.get(expert_id=expert_id)
        except Expert.DoesNotExist:
            return Response({'detail': 'expert not found'}, status=status.HTTP_404_NOT_FOUND)

        if op == 'add':
            VideoAssignment.objects.get_or_create(video=video, expert=expert, defaults={'source': 'admin'})
        else:
            VideoAssignment.objects.filter(video=video, expert=expert).delete()

        assigned = VideoAssignment.objects.filter(video=video).select_related('expert')
        return Response({
            'success': True,
            'video_id': video_id,
            'assigned_experts': [
                {'expert_id': a.expert_id, 'display_name': a.expert.display_name or a.expert_id}
                for a in assigned
            ],
        })
