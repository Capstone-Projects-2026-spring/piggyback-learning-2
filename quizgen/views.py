from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView

from videos.models import Video
from videos.services.download import download_youtube
from videos.services.frames import extract_frames_per_second_for_video

from .models import SubmittedQuestionSet


class AdminVideosAPIView(APIView):
    """
    GET /api/admin/videos?include_without_frames=false
    The old FastAPI scanned downloads/. Now: query DB.
    """

    authentication_classes = []
    permission_classes = []

    def get(self, request):
        include_without_frames = request.query_params.get(
            'include_without_frames', 'false'
        ).lower() in {'1', 'true', 'yes'}

        qs = Video.objects.all().order_by('-created_at')

        videos = []
        for v in qs:
            has_frames = v.frames.exists()
            if not include_without_frames and not has_frames:
                continue

            has_questions = SubmittedQuestionSet.objects.filter(video=v).exists()
            videos.append(
                {
                    'video_id': v.id,
                    'title': v.title or v.id,
                    'duration_seconds': v.duration_seconds,
                    'duration_formatted': None,  # optional; add if you want HH:MM:SS
                    'has_frames': has_frames,
                    'frame_count': v.frames.count() if has_frames else None,
                    'has_questions': has_questions,
                    'frames_dir': None,  # no longer a folder path; DB-backed
                    'question_file': None,  # no longer a file; DB-backed
                }
            )

        return Response(
            {
                'success': True,
                'count': len(videos),
                'videos': videos,
                'message': (
                    'Videos with extracted frames ready for question generation.'
                    if not include_without_frames
                    else 'All downloaded videos.'
                ),
            }
        )


class SubmitQuestionsAPIView(APIView):
    """
    POST /api/submit-questions
    FastAPI wrote downloads/<id>/questions/<id>.json.
    Now stores a SubmittedQuestionSet(payload=...) in DB.
    """

    authentication_classes = []
    permission_classes = []

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        video_id = payload.get('video_id')
        questions_data = payload.get('questions', [])

        if not video_id or not isinstance(questions_data, list) or not questions_data:
            return Response(
                {'detail': 'Missing video_id or questions'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        video, _ = Video.objects.get_or_create(id=video_id)

        saved = SubmittedQuestionSet.objects.create(
            video=video,
            status='submitted',
            payload={
                'video_id': video_id,
                'status': 'submitted',
                'segments': questions_data,
            },
        )

        return Response(
            {
                'success': True,
                'message': 'Questions submitted successfully',
                'submitted_set_id': saved.id,
            },
            status=status.HTTP_201_CREATED,
        )


class DownloadAPIView(APIView):
    authentication_classes = []
    permission_classes = []

    def post(self, request):
        url = request.data.get('url')
        if not url:
            return Response(
                {'detail': 'Missing url'}, status=status.HTTP_400_BAD_REQUEST
            )

        result = download_youtube(url)
        http_status = (
            status.HTTP_200_OK if result.get('success') else status.HTTP_400_BAD_REQUEST
        )
        return Response(result, status=http_status)


class ExtractFramesAPIView(APIView):
    authentication_classes = []
    permission_classes = []

    def post(self, request, video_id: str):
        result = extract_frames_per_second_for_video(video_id)
        http_status = (
            status.HTTP_200_OK if result.get('success') else status.HTTP_400_BAD_REQUEST
        )
        return Response(result, status=http_status)
