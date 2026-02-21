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

    # store relative path like: downloads/<video_id>/<file>.mp4 (or /downloads/... if you prefer)
    local_video_path = models.CharField(max_length=1024, blank=True, default='')

    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    def __str__(self) -> str:
        return f'{self.id} - {self.title or "Untitled"}'


class VideoAsset(models.Model):
    """
    Optional but useful: track the actual downloaded files in downloads/<video_id>/.
    (Your FastAPI code enumerates these in multiple places.)
    """

    video = models.ForeignKey(Video, on_delete=models.CASCADE, related_name='assets')
    file_path = models.CharField(max_length=1024)  # relative under downloads/
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
    """
    Replaces downloads/<video_id>/extracted_frames/frame_data.json (+ csv),
    and supports optional Subtitle_Text in your CSV reader.
    """

    video = models.ForeignKey(Video, on_delete=models.CASCADE, related_name='frames')

    # aligns with frame_data.json keys
    frame_number = models.IntegerField()
    timestamp_seconds = models.IntegerField(db_index=True)
    timestamp_formatted = models.CharField(
        max_length=16, blank=True, default=''
    )  # "MM:SS"
    filename = models.CharField(max_length=255)  # "frame_0001s.jpg"
    file_path = models.CharField(max_length=1024)  # relative under downloads/

    subtitle_text = models.TextField(blank=True, default='')  # may be absent

    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        unique_together = [('video', 'timestamp_seconds')]
        indexes = [
            models.Index(fields=['video', 'timestamp_seconds']),
        ]
