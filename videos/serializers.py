from rest_framework import serializers

from .models import Video


class VideoSerializer(serializers.ModelSerializer):
    video_id = serializers.CharField(source='id')
    local_path = serializers.CharField(source='local_video_path')
    thumbnail = serializers.CharField(source='thumbnail_url')

    class Meta:
        model = Video
        fields = ('video_id', 'title', 'duration_seconds', 'local_path', 'thumbnail')
