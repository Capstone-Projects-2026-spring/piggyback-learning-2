from django.db import models

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
    video = models.ForeignKey(
        Video, on_delete=models.CASCADE, related_name='generation_jobs'
    )
    start_offset_seconds = models.IntegerField(default=0)
    interval_seconds = models.IntegerField(default=60)
    duration_seconds = models.IntegerField(null=True, blank=True)
    full_duration = models.BooleanField(default=False)
    created_at = models.DateTimeField(auto_now_add=True)


class Segment(models.Model):
    video = models.ForeignKey(Video, on_delete=models.CASCADE, related_name='segments')
    start_seconds = models.IntegerField()
    end_seconds = models.IntegerField()
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
    segment = models.OneToOneField(
        Segment, on_delete=models.CASCADE, related_name='llm_result'
    )
    raw = models.JSONField(default=dict, blank=True)
    best_question = models.TextField(blank=True, default='')
    has_error = models.BooleanField(default=False)
    error = models.JSONField(default=dict, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)


class GeneratedQuestion(models.Model):
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
    video = models.ForeignKey(
        Video, on_delete=models.CASCADE, related_name='submitted_sets'
    )
    status = models.CharField(max_length=32, default='submitted')
    submitted_at = models.DateTimeField(auto_now_add=True)
    payload = models.JSONField(default=dict, blank=True)


class QuizScore(models.Model):
    """
    Stores quiz results for a child after completing a video.
    Replaces downloads/quiz_results/ JSON files.
    """

    child_id = models.CharField(max_length=64, db_index=True)
    video_id = models.CharField(max_length=64, db_index=True)
    score_data = models.JSONField(default=dict, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        indexes = [models.Index(fields=['child_id', 'video_id'])]

    def __str__(self):
        return f'QuizScore child={self.child_id} video={self.video_id}'
