from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView

from quizgen.serializers import SubmittedQuestionSetSerializer
from videos.models import Video
from videos.services.download import download_youtube
from videos.services.frames import extract_frames_per_second_for_video

from .models import QuizScore, SubmittedQuestionSet


class AdminVideosAPIView(APIView):
    """GET /api/admin/videos?include_without_frames=false"""

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
                    'duration_formatted': None,
                    'has_frames': has_frames,
                    'frame_count': v.frames.count() if has_frames else None,
                    'has_questions': has_questions,
                    'frames_dir': None,
                    'question_file': None,
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
    """POST /api/submit-questions"""

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


class VideoQuestionsAPIView(APIView):
    authentication_classes = []
    permission_classes = []
    serializer_class = SubmittedQuestionSetSerializer

    def get(self, request, video_id):
        qs = SubmittedQuestionSet.objects.filter(video__id=video_id)
        serializer = SubmittedQuestionSetSerializer(qs, many=True)
        return Response(serializer.data)


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


# ── Quiz scores ────────────────────────────────────────────────────────────────

class SaveQuizScoreAPIView(APIView):
    """POST /api/save-quiz-score"""

    authentication_classes = []
    permission_classes = []

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        child_id = payload.get('child_id')
        video_id = payload.get('video_id')
        score_data = payload.get('score_data', {})

        if not child_id or not video_id:
            return Response(
                {'success': False, 'message': 'Missing child_id or video_id'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        QuizScore.objects.create(
            child_id=child_id,
            video_id=video_id,
            score_data=score_data,
        )

        return Response({'success': True}, status=status.HTTP_201_CREATED)


class GetQuizScoresAPIView(APIView):
    """GET /api/get-quiz-scores/{child_id}"""

    authentication_classes = []
    permission_classes = []

    def get(self, request, child_id: str):
        scores = QuizScore.objects.filter(child_id=child_id).order_by('-created_at')
        results = [
            {
                'child_id': s.child_id,
                'video_id': s.video_id,
                'score_data': s.score_data,
                'created_at': s.created_at.isoformat(),
            }
            for s in scores
        ]
        return Response({'success': True, 'scores': results})
