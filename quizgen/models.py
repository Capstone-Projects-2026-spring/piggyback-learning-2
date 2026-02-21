from django.db import models
from django.db.models import JSONField

from videos.models import Video


class QuestionType(models.TextChoices):
    CHARACTER = 'character', 'Character'
    SETTING = 'setting', 'Setting'
    FEELING = 'feeling', 'Feeling'
    ACTION = 'action', 'Action'
    CAUSAL = 'causal', 'Causal'
    OUTCOME = 'outcome', 'Outcome'
    PREDICTION = 'prediction', 'Prediction'


class GenerationJob(models.Model):
    """
    Represents a “run” over either a single interval or full-duration segmentation.
    This mirrors the websocket aggregation payload shape.
    """

    video = models.ForeignKey(
        Video, on_delete=models.CASCADE, related_name='generation_jobs'
    )
    start_offset_seconds = models.IntegerField(default=0)
    interval_seconds = models.IntegerField(default=60)
    duration_seconds = models.IntegerField(
        null=True, blank=True
    )  # known when full_duration=True

    full_duration = models.BooleanField(default=False)
    created_at = models.DateTimeField(auto_now_add=True)


class Segment(models.Model):
    """
    One (start,end) window for a video. Used by both generation + review flows.
    """

    video = models.ForeignKey(Video, on_delete=models.CASCADE, related_name='segments')
    start_seconds = models.IntegerField()
    end_seconds = models.IntegerField()

    # optional link to a GenerationJob
    job = models.ForeignKey(
        GenerationJob,
        on_delete=models.SET_NULL,
        null=True,
        blank=True,
        related_name='segments',
    )

    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        unique_together = [('video', 'start_seconds', 'end_seconds')]
        indexes = [models.Index(fields=['video', 'start_seconds', 'end_seconds'])]


class SegmentLLMResult(models.Model):
    """
    Stores the raw JSON object returned by the model for a segment,
    plus structured questions for querying.
    (Your FastAPI stores raw JSON in files; this replaces that.)
    """

    segment = models.OneToOneField(
        Segment, on_delete=models.CASCADE, related_name='llm_result'
    )

    # raw response (or error payload) from _maybe_parse_json/_wrap_segment_result
    raw = JSONField(default=dict, blank=True)

    best_question = models.TextField(blank=True, default='')

    # if generation failed, capture the normalized error + debug
    has_error = models.BooleanField(default=False)
    error = JSONField(default=dict, blank=True)

    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)


class GeneratedQuestion(models.Model):
    """
    Normalized questions per type for a segment.
    Matches the schema:
      questions: { character: {q,a,rank}, ... }
    """

    segment = models.ForeignKey(
        Segment, on_delete=models.CASCADE, related_name='generated_questions'
    )
    qtype = models.CharField(max_length=32, choices=QuestionType.choices)
    question = models.TextField()
    answer = models.TextField()
    llm_rank = models.IntegerField(null=True, blank=True)

    class Meta:
        unique_together = [('segment', 'qtype')]


class SubmittedQuestionSet(models.Model):
    """
    Replaces downloads/<video_id>/questions/<video_id>.json created by /submit-questions.
    Stores the admin “finalized questions” bundle.
    """

    video = models.ForeignKey(
        Video, on_delete=models.CASCADE, related_name='submitted_sets'
    )
    status = models.CharField(max_length=32, default='submitted')
    submitted_at = models.DateTimeField(auto_now_add=True)

    # store exactly what the UI submits (segments array etc.)
    payload = JSONField(default=dict, blank=True)
