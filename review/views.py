from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView
from quizgen.models import Segment, SubmittedQuestionSet
import json

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
        video_id = payload.get('video_id') or payload.get('videoId')
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
        questions = payload.get('questions', []) 

        if not video_id:
            return Response(
                {'detail': 'Missing video_id'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        if not isinstance(questions, list):
            return Response(
                {'detail': 'questions must be a list'},
                status=status.HTTP_400_BAD_REQUEST,
            )

        video, _ = Video.objects.get_or_create(id=video_id)

        final_set = FinalQuestionSet.objects.create(
            video=video,
            payload=payload
        )

        for item in questions:
            if not isinstance(item, dict):
                continue

            start = int(item.get('start') or 0)
            end = int(item.get('end') or 0)

            fs = FinalSegment.objects.create(
                final_set=final_set,
                segment_index=None,
                start_seconds=start,
                end_seconds=end,
            )

            result = item.get('result') or {}
            questions_dict = result.get('questions') or {}

            if not isinstance(questions_dict, dict):
                continue

            for qtype, qdata in questions_dict.items():
                if not isinstance(qdata, dict):
                    continue

                question = (qdata.get('q') or '').strip()
                answer = (qdata.get('a') or '').strip()

                if not question:
                    continue

                followup = qdata.get('followup') or {}
                followup_q = (followup.get('q') or '').strip()
                followup_a = (followup.get('a') or '').strip()

                try:
                    rank = int(qdata.get('rank'))
                except Exception:
                    rank = None

                FinalAIQuestion.objects.create(
                    final_segment=fs,
                    qtype=(qtype or '').strip().lower() or 'character',
                    question=question,
                    answer=answer,
                    llm_ranking=rank,
                    expert_ranking=None,
                    trashed=False,

                    followup_question=followup_q,
                    followup_answer=followup_a,
                )

        return Response(
            {'success': True, 'final_set_id': final_set.id},
            status=status.HTTP_201_CREATED,
        )

class FinalQuestionsForKidsAPIView(APIView):
    authentication_classes = []
    permission_classes = []

    def get(self, request, video_id: str):
        # ---------------------------------------------------------
        # Path 1: Check for an explicitly saved FinalQuestionSet
        # ---------------------------------------------------------
        final_set = (
            FinalQuestionSet.objects.filter(video_id=video_id)
            .order_by('-saved_at')
            .first()
        )

        if final_set:
            selected_segments = []
            segments = final_set.segments.all().order_by('start_seconds', 'end_seconds')

            for seg in segments:
                qs = seg.ai_questions.filter(trashed=False).order_by('llm_ranking', 'id')
                best = qs.first()
                if not best:
                    continue

                selected_segments.append({
                    'segment_range_start': seg.start_seconds,
                    'segment_range_end': seg.end_seconds,
                    'question': best.question,
                    'answer': best.answer,
                    'llm_ranking': best.llm_ranking,
                    'expert_ranking': best.expert_ranking,
                    'followup_question': best.followup_question,
                    'followup_answer': best.followup_answer,
                })

            # THE FIX: Only return this data if we actually found valid questions!
            # If selected_segments is empty, it will skip this return and fall back 
            # to your SubmittedQuestionSet JSON data below.
            if len(selected_segments) > 0:
                return Response({'success': True, 'segments': selected_segments})

        # ---------------------------------------------------------
        # Path 2: Fallback to SubmittedQuestionSet (Your JSON example)
        # ---------------------------------------------------------
        submitted_set = (
            SubmittedQuestionSet.objects.filter(video_id=video_id)
            .order_by('-id')
            .first()
        )

        if submitted_set:
            # BUG FIX 1: Safely handle the payload whether it's a Dictionary OR a JSON String
            raw_payload = submitted_set.payload
            if isinstance(raw_payload, str):
                try:
                    payload = json.loads(raw_payload)
                except json.JSONDecodeError:
                    payload = {}
            elif isinstance(raw_payload, dict):
                payload = raw_payload
            else:
                payload = {}

            segments = payload.get('segments', [])
            selected_segments = []

            for seg in segments:
                if not isinstance(seg, dict):
                    continue

                start = seg.get('start', 0)
                end = seg.get('end', 0)

                result = seg.get('result') or {}
                questions = result.get('questions') or {}

                if not isinstance(questions, dict) or not questions:
                    continue

                # BUG FIX 2: Ignore the faulty "best_question" string entirely. 
                # Instead, just sort all available questions by their LLM rank and grab the #1 question.
                ranked_questions = []
                for _, qdata in questions.items():
                    if not isinstance(qdata, dict):
                        continue
                    try:
                        rank = int(qdata.get('rank', 999999))
                    except Exception:
                        rank = 999999
                    ranked_questions.append((rank, qdata))

                if not ranked_questions:
                    continue

                # Sort by rank and pick the best one
                ranked_questions.sort(key=lambda x: x[0])
                best_q = ranked_questions[0][1]

                question = (best_q.get('q') or '').strip()
                answer = (best_q.get('a') or '').strip()

                if not question or not answer:
                    continue

                # Safely extract followups
                followup = best_q.get('followup') or {}
                followup_question = (followup.get('q') or '').strip()
                followup_answer = (followup.get('a') or '').strip()

                selected_segments.append({
                    'segment_range_start': start,
                    'segment_range_end': end,
                    'question': question,
                    'answer': answer,
                    'llm_ranking': best_q.get('rank'),
                    'expert_ranking': None,
                    'followup_question': followup_question,
                    'followup_answer': followup_answer,
                })

            # BUG FIX 3: Removed the `selected_segments[:-1]` code here too!
            return Response({'success': True, 'segments': selected_segments})

        # If neither exists
        return Response(
            {'success': False, 'error': 'final_questions not found'},
            status=404,
        )
