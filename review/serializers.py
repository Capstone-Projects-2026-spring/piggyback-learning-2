from rest_framework import serializers

from .models import (
    ExpertAnnotation,
    ExpertQuestion,
    FinalAIQuestion,
    FinalQuestionSet,
    FinalSegment,
)


class ExpertAnnotationSerializer(serializers.ModelSerializer):
    class Meta:
        model = ExpertAnnotation
        fields = '__all__'


class ExpertQuestionSerializer(serializers.ModelSerializer):
    class Meta:
        model = ExpertQuestion
        fields = '__all__'


class FinalAIQuestionSerializer(serializers.ModelSerializer):
    class Meta:
        model = FinalAIQuestion
        fields = '__all__'


class FinalSegmentSerializer(serializers.ModelSerializer):
    ai_questions = FinalAIQuestionSerializer(many=True, read_only=True)

    class Meta:
        model = FinalSegment
        fields = ('id', 'segment_index', 'start_seconds', 'end_seconds', 'ai_questions')


class FinalQuestionSetSerializer(serializers.ModelSerializer):
    segments = FinalSegmentSerializer(many=True, read_only=True)

    class Meta:
        model = FinalQuestionSet
        fields = ('id', 'video', 'saved_at', 'payload', 'segments')
