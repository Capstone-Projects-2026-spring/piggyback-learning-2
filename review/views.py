from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView

from quizgen.models import Segment
from videos.models import Video

from .models import (
    ExpertAnnotation,
    ExpertQuestion,
    FinalAIQuestion,
    FinalQuestionSet,
    FinalSegment,
)


class SaveExpertAnnotationAPIView(APIView):
    """
    POST /api/expert-annotations
    Mirrors FastAPI: it upserts an annotation by (start,end,mode).
    Now: save ExpertAnnotation row in DB.

    Expected payload (same as FastAPI):
      - mode: "review" or "create"
      - video_id (required for create)
      - start, end (required)
      - skip (bool)
      - segment_index (optional)
      - question_type, question, answer (if not skipped)
      - best_question {question, answer, approved, comment} (review mode only)
    """

    authentication_classes = []
    permission_classes = []

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        mode = (payload.get('mode') or 'review').strip()

        video_id = payload.get('video_id')
        start = payload.get('start')
        end = payload.get('end')

        if not video_id:
            return Response(
                {'detail': 'Missing video_id'}, status=status.HTTP_400_BAD_REQUEST
            )
        if start is None or end is None:
            return Response(
                {'detail': 'Missing start/end'}, status=status.HTTP_400_BAD_REQUEST
            )

        try:
            start_i = int(start)
            end_i = int(end)
        except Exception:
            return Response(
                {'detail': 'Invalid start/end'}, status=status.HTTP_400_BAD_REQUEST
            )

        video, _ = Video.objects.get_or_create(id=video_id)

        # Ensure Segment exists (used as anchor)
        seg, _ = Segment.objects.get_or_create(
            video=video,
            start_seconds=start_i,
            end_seconds=end_i,
            defaults={},
        )

        skip_requested = bool(payload.get('skip'))
        segment_index = payload.get('segment_index')
        try:
            segment_index_i = int(segment_index) if segment_index is not None else None
        except Exception:
            segment_index_i = None

        defaults = {
            'video': video,
            'mode': mode,
            'segment_index': segment_index_i,
            'skipped': skip_requested,
        }

        if skip_requested:
            defaults.update(
                {
                    'question_type': 'skip',
                    'question': '(skipped)',
                    'answer': '',
                    'best_question': {},
                }
            )
        else:
            qtype = (payload.get('question_type') or '').strip().lower()
            question = (payload.get('question') or '').strip()
            answer = (payload.get('answer') or '').strip()

            if not question or not answer:
                return Response(
                    {'detail': 'Question and answer are required.'},
                    status=status.HTTP_400_BAD_REQUEST,
                )
            if not qtype:
                return Response(
                    {'detail': 'Missing question_type.'},
                    status=status.HTTP_400_BAD_REQUEST,
                )

            defaults.update(
                {
                    'question_type': qtype,
                    'question': question,
                    'answer': answer,
                }
            )

            # best_question only relevant for review mode
            if mode == 'review' and isinstance(payload.get('best_question'), dict):
                defaults['best_question'] = payload['best_question']

        obj, created = ExpertAnnotation.objects.update_or_create(
            segment=seg,
            mode=mode,
            defaults=defaults,
        )

        return Response(
            {
                'success': True,
                'updated': not created,
                'mode': mode,
                'annotation': {
                    'id': obj.id,
                    'video_id': video.id,
                    'start': start_i,
                    'end': end_i,
                    'segment_index': obj.segment_index,
                    'skipped': obj.skipped,
                    'question_type': obj.question_type,
                    'question': obj.question,
                    'answer': obj.answer,
                    'best_question': obj.best_question,
                    'saved_at': obj.saved_at,
                },
            }
        )


class ExpertQuestionsByVideoAPIView(APIView):
    """
    GET /api/expert-questions/{video_id}
    FastAPI returned the expert_questions.json list.
    Here: return DB ExpertQuestion rows for that video.
    """

    authentication_classes = []
    permission_classes = []

    def get(self, request, video_id: str):
        qs = ExpertQuestion.objects.filter(video_id=video_id).order_by(
            'segment_start', 'segment_end'
        )
        items = []
        for q in qs:
            items.append(
                {
                    'segmentStart': q.segment_start,
                    'segmentEnd': q.segment_end,
                    'timestamp': q.timestamp,
                    'skipped': q.skipped,
                    'skipReason': q.skip_reason,
                    'questionType': q.question_type,
                    'question': q.question,
                    'answer': q.answer,
                }
            )
        return Response({'success': True, 'video_id': video_id, 'questions': items})


class SaveExpertQuestionsAPIView(APIView):
    """
    POST /api/expert-questions
    Accepts {video_id, questions:[...]} or a single question object.
    Stores to ExpertQuestion table.
    """

    authentication_classes = []
    permission_classes = []

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        video_id = payload.get('video_id')
        if not video_id:
            return Response(
                {'detail': 'Missing video_id'}, status=status.HTTP_400_BAD_REQUEST
            )

        video, _ = Video.objects.get_or_create(id=video_id)

        questions = payload.get('questions')
        if questions is None:
            # allow single question POST
            questions = [payload]

        if not isinstance(questions, list):
            return Response(
                {'detail': 'questions must be a list'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        saved = 0
        for item in questions:
            if not isinstance(item, dict):
                continue

            seg_start = float(
                item.get('segmentStart') or item.get('segment_start') or 0.0
            )
            seg_end = float(item.get('segmentEnd') or item.get('segment_end') or 0.0)
            timestamp = float(item.get('timestamp') or 0.0)

            skipped = bool(item.get('skipped', False))
            skip_reason = (
                item.get('skipReason') or item.get('skip_reason') or ''
            ).strip()
            qtype = (
                (item.get('questionType') or item.get('question_type') or '')
                .strip()
                .lower()
            )
            question = (item.get('question') or '').strip()
            answer = (item.get('answer') or '').strip()

            ExpertQuestion.objects.update_or_create(
                video=video,
                segment_start=seg_start,
                segment_end=seg_end,
                defaults={
                    'timestamp': timestamp,
                    'skipped': skipped,
                    'skip_reason': skip_reason,
                    'question_type': qtype,
                    'question': question,
                    'answer': answer,
                },
            )
            saved += 1

        return Response(
            {'success': True, 'saved': saved}, status=status.HTTP_201_CREATED
        )


class SaveFinalQuestionsAPIView(APIView):
    """
    POST /api/save-final-questions
    FastAPI saved final_questions.json. Here we store FinalQuestionSet + normalized tables.

    Expected payload shape (from your UI):
      { video_id, segments:[ {start,end, segmentIndex?, aiQuestions:[...]} ] }
    """

    authentication_classes = []
    permission_classes = []

    def post(self, request):
        payload = request.data if isinstance(request.data, dict) else {}
        video_id = payload.get('video_id')
        segments = payload.get('segments', [])

        if not video_id:
            return Response(
                {'detail': 'Missing video_id'}, status=status.HTTP_400_BAD_REQUEST
            )
        if not isinstance(segments, list):
            return Response(
                {'detail': 'segments must be a list'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        video, _ = Video.objects.get_or_create(id=video_id)

        final_set = FinalQuestionSet.objects.create(video=video, payload=payload)

        # Normalize
        for seg in segments:
            if not isinstance(seg, dict):
                continue
            start = int(seg.get('start') or 0)
            end = int(seg.get('end') or 0)
            seg_index = seg.get('segmentIndex', seg.get('segment_index'))
            try:
                seg_index = int(seg_index) if seg_index is not None else None
            except Exception:
                seg_index = None

            fs = FinalSegment.objects.create(
                final_set=final_set,
                segment_index=seg_index,
                start_seconds=start,
                end_seconds=end,
            )

            ai_qs = seg.get('aiQuestions') or seg.get('ai_questions') or []
            if isinstance(ai_qs, list):
                for q in ai_qs:
                    if not isinstance(q, dict):
                        continue
                    FinalAIQuestion.objects.create(
                        final_segment=fs,
                        qtype=(q.get('questionType') or q.get('qtype') or '')
                        .strip()
                        .lower()
                        or 'character',
                        question=(
                            q.get('question') or q.get('originalQuestion') or ''
                        ).strip(),
                        answer=(
                            q.get('answer') or q.get('originalAnswer') or ''
                        ).strip(),
                        llm_ranking=q.get('llm_ranking'),
                        expert_ranking=q.get('expert_ranking'),
                        trashed=bool(q.get('trashed', False)),
                    )

        return Response(
            {'success': True, 'final_set_id': final_set.id},
            status=status.HTTP_201_CREATED,
        )


class FinalQuestionsForKidsAPIView(APIView):
    """
    GET /api/final-questions/{video_id}
    Mirrors FastAPI behavior:
      - choose best LLM-ranked question per segment (lowest llm_ranking)
      - skip trashed
      - exclude the final segment (drops last item)
    """

    authentication_classes = []
    permission_classes = []

    def get(self, request, video_id: str):
        final_set = (
            FinalQuestionSet.objects.filter(video_id=video_id)
            .order_by('-saved_at')
            .first()
        )
        if not final_set:
            return Response(
                {'success': False, 'error': 'final_questions not found'}, status=404
            )

        selected_segments = []
        segments = final_set.segments.all().order_by('start_seconds', 'end_seconds')

        for seg in segments:
            qs = seg.ai_questions.filter(trashed=False).order_by('llm_ranking', 'id')
            best = qs.first()
            if not best:
                continue
            selected_segments.append(
                {
                    'segment_range_start': seg.start_seconds,
                    'segment_range_end': seg.end_seconds,
                    'question': best.question,
                    'answer': best.answer,
                    'llm_ranking': best.llm_ranking,
                    'expert_ranking': best.expert_ranking,
                }
            )

        if selected_segments:
            selected_segments = selected_segments[:-1]

        return Response({'success': True, 'segments': selected_segments})
