from django.contrib import admin

from .models import ExtractedFrame, Video, VideoAsset


class VideoAssetInline(admin.TabularInline):
    model = VideoAsset
    extra = 0
    fields = ('kind', 'file_path', 'created_at')
    readonly_fields = ('created_at',)
    show_change_link = True


class ExtractedFrameInline(admin.TabularInline):
    model = ExtractedFrame
    extra = 0
    fields = (
        'timestamp_seconds',
        'timestamp_formatted',
        'frame_number',
        'filename',
        'file_path',
        'subtitle_text',
        'created_at',
    )
    readonly_fields = ('created_at',)
    show_change_link = True


@admin.register(Video)
class VideoAdmin(admin.ModelAdmin):
    list_display = ('id', 'title', 'duration_seconds', 'created_at', 'updated_at')
    search_fields = ('id', 'title')
    list_filter = ('created_at', 'updated_at')
    readonly_fields = ('created_at', 'updated_at')
    inlines = [VideoAssetInline]
    ordering = ('-created_at',)


@admin.register(VideoAsset)
class VideoAssetAdmin(admin.ModelAdmin):
    list_display = ('id', 'video', 'kind', 'file_path', 'created_at')
    search_fields = ('video__id', 'video__title', 'file_path')
    list_filter = ('kind', 'created_at')
    readonly_fields = ('created_at',)
    ordering = ('-created_at',)


@admin.register(ExtractedFrame)
class ExtractedFrameAdmin(admin.ModelAdmin):
    list_display = (
        'id',
        'video',
        'timestamp_seconds',
        'frame_number',
        'filename',
        'created_at',
    )
    search_fields = (
        'video__id',
        'video__title',
        'filename',
        'file_path',
        'subtitle_text',
    )
    list_filter = ('created_at',)
    readonly_fields = ('created_at',)
    ordering = ('video', 'timestamp_seconds')
