from django.db import models


class Video(models.Model):
    """
    Replaces downloads/<video_id>/meta.json and provides the anchor FK
    for everything else.
    """

    id = models.CharField(primary_key=True, max_length=64)  # video_id from yt-dlp
    title = models.CharField(max_length=500, blank=True, default='')
    thumbnail_url = models.URLField(blank=True, default='')
    duration_seconds = models.IntegerField(null=True, blank=True)

    local_video_path = models.CharField(max_length=1024, blank=True, default='')

    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    def __str__(self) -> str:
        return f'{self.id} - {self.title or "Untitled"}'


class VideoAsset(models.Model):
    video = models.ForeignKey(Video, on_delete=models.CASCADE, related_name='assets')
    file_path = models.CharField(max_length=1024)
    kind = models.CharField(
        max_length=32,
        choices=[
            ('video', 'video'),
            ('subtitle', 'subtitle'),
            ('thumbnail', 'thumbnail'),
            ('meta', 'meta'),
            ('other', 'other'),
        ],
        default='other',
    )
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        unique_together = [('video', 'file_path')]


class ExtractedFrame(models.Model):
    video = models.ForeignKey(Video, on_delete=models.CASCADE, related_name='frames')

    frame_number = models.IntegerField()
    timestamp_seconds = models.IntegerField(db_index=True)
    timestamp_formatted = models.CharField(max_length=16, blank=True, default='')
    filename = models.CharField(max_length=255)
    file_path = models.CharField(max_length=1024)

    subtitle_text = models.TextField(blank=True, default='')

    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        unique_together = [('video', 'timestamp_seconds')]
        indexes = [
            models.Index(fields=['video', 'timestamp_seconds']),
        ]


class VideoAssignment(models.Model):
    """
    Many-to-many between Video and Expert.
    Replaces the SQLite expert_video_assignments table.
    """

    video = models.ForeignKey(Video, on_delete=models.CASCADE, related_name='assignments')
    # Use string reference to avoid circular import with user app
    expert = models.ForeignKey(
        'user.Expert', on_delete=models.CASCADE, related_name='video_assignments'
    )
    claimed = models.BooleanField(default=False)
    source = models.CharField(max_length=32, default='admin')
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        unique_together = [('video', 'expert')]

    def __str__(self):
        return f'{self.expert_id} -> {self.video_id}'
