from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView

from .models import Video
from .serializers import VideoSerializer


def _format_mmss(sec: int | None) -> str:
    if not sec or sec < 0:
        return '00:00'
    m, s = divmod(int(sec), 60)
    return f'{m:02d}:{s:02d}'


class KidsVideosAPIView(APIView):
    """
    GET /api/kids_videos
    Mirrors the FastAPI shape: {success, count, videos:[...]}
    """

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
    """
    GET /api/video-list

    FastAPI-equivalent endpoint to list downloaded videos.
    """

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
