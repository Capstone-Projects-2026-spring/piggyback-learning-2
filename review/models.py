from django.db import models
from django.db.models import JSONField

from quizgen.models import QuestionType, Segment
from videos.models import Video


class ReviewMode(models.TextChoices):
    REVIEW = 'review', 'Review'
    CREATE = 'create', 'Create'


class ExpertAnnotation(models.Model):
    """
    Replaces <question_file>.expert.json entries (the “annotations” list).
    Anchored to a segment.
    """

    video = models.ForeignKey(
        Video, on_delete=models.CASCADE, related_name='expert_annotations'
    )
    segment = models.ForeignKey(
        Segment, on_delete=models.CASCADE, related_name='expert_annotations'
    )

    mode = models.CharField(
        max_length=16, choices=ReviewMode.choices, default=ReviewMode.REVIEW
    )

    segment_index = models.IntegerField(null=True, blank=True)

    skipped = models.BooleanField(default=False)

    # When skipped=True, your FastAPI uses question_type="skip"
    question_type = models.CharField(
        max_length=32,
        choices=[('skip', 'skip')] + list(QuestionType.choices),
        default='skip',
    )

    question = models.TextField(blank=True, default='')
    answer = models.TextField(blank=True, default='')

    # best_question object {question, answer, approved, comment}
    best_question = JSONField(default=dict, blank=True)

    saved_at = models.DateTimeField(auto_now=True)

    class Meta:
        unique_together = [('segment', 'mode')]


class ExpertQuestion(models.Model):
    """
    Replaces downloads/<video_id>/expert_questions/expert_questions.json
    (different schema than ExpertAnnotation).
    """

    video = models.ForeignKey(
        Video, on_delete=models.CASCADE, related_name='expert_questions'
    )

    segment_start = models.FloatField()
    segment_end = models.FloatField()
    timestamp = models.FloatField()

    skipped = models.BooleanField(default=False)
    skip_reason = models.TextField(blank=True, default='')

    question_type = models.CharField(
        max_length=32, choices=QuestionType.choices, blank=True, default=''
    )
    question = models.TextField(blank=True, default='')
    answer = models.TextField(blank=True, default='')

    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        indexes = [models.Index(fields=['video', 'segment_start', 'segment_end'])]


class FinalQuestionSet(models.Model):
    """
    Replaces downloads/<video_id>/final_questions/final_questions.json
    """

    video = models.ForeignKey(
        Video, on_delete=models.CASCADE, related_name='final_sets'
    )
    saved_at = models.DateTimeField(auto_now_add=True)

    # store the full blob for perfect fidelity with your current payloads
    payload = JSONField(default=dict, blank=True)


class FinalSegment(models.Model):
    """
    Normalized segments inside a FinalQuestionSet.
    """

    final_set = models.ForeignKey(
        FinalQuestionSet, on_delete=models.CASCADE, related_name='segments'
    )
    segment_index = models.IntegerField(null=True, blank=True)
    start_seconds = models.IntegerField()
    end_seconds = models.IntegerField()

    class Meta:
        unique_together = [('final_set', 'start_seconds', 'end_seconds')]


class FinalAIQuestion(models.Model):
    """
    Normalized aiQuestions entries inside a final segment.
    """

    final_segment = models.ForeignKey(
        FinalSegment, on_delete=models.CASCADE, related_name='ai_questions'
    )

    qtype = models.CharField(max_length=32, choices=QuestionType.choices)
    question = models.TextField(blank=True, default='')
    answer = models.TextField(blank=True, default='')

    llm_ranking = models.IntegerField(null=True, blank=True)
    expert_ranking = models.IntegerField(null=True, blank=True)

    trashed = models.BooleanField(default=False)
