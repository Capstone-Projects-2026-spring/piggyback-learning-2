from rest_framework import serializers

from .models import GeneratedQuestion, Segment, SegmentLLMResult, SubmittedQuestionSet


class GeneratedQuestionSerializer(serializers.ModelSerializer):
    class Meta:
        model = GeneratedQuestion
        fields = ('qtype', 'question', 'answer', 'llm_rank')


class SegmentLLMResultSerializer(serializers.ModelSerializer):
    generated_questions = GeneratedQuestionSerializer(many=True, read_only=True)

    class Meta:
        model = SegmentLLMResult
        fields = ('raw', 'best_question', 'has_error', 'error', 'generated_questions')


class SegmentSerializer(serializers.ModelSerializer):
    llm_result = SegmentLLMResultSerializer(read_only=True)

    class Meta:
        model = Segment
        fields = (
            'id',
            'video',
            'start_seconds',
            'end_seconds',
            'job',
            'created_at',
            'llm_result',
        )


class SubmittedQuestionSetSerializer(serializers.ModelSerializer):
    class Meta:
        model = SubmittedQuestionSet
        fields = ('id', 'video', 'status', 'submitted_at', 'payload')
