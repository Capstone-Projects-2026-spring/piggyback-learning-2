from django.contrib import admin

from .models import (
    GeneratedQuestion,
    GenerationJob,
    Segment,
    SegmentLLMResult,
    SubmittedQuestionSet,
)


class SegmentInline(admin.TabularInline):
    model = Segment
    extra = 0
    fields = ('video', 'start_seconds', 'end_seconds', 'created_at')
    readonly_fields = ('created_at',)
    show_change_link = True


@admin.register(GenerationJob)
class GenerationJobAdmin(admin.ModelAdmin):
    list_display = (
        'id',
        'video',
        'full_duration',
        'start_offset_seconds',
        'interval_seconds',
        'duration_seconds',
        'created_at',
    )
    search_fields = ('video__id', 'video__title')
    list_filter = ('full_duration', 'created_at')
    readonly_fields = ('created_at',)
    inlines = [SegmentInline]
    ordering = ('-created_at',)


class GeneratedQuestionInline(admin.TabularInline):
    model = GeneratedQuestion
    extra = 0
    fields = ('qtype', 'llm_rank', 'question', 'answer')
    show_change_link = True


@admin.register(Segment)
class SegmentAdmin(admin.ModelAdmin):
    list_display = ('id', 'video', 'start_seconds', 'end_seconds', 'job', 'created_at')
    search_fields = ('video__id', 'video__title')
    list_filter = ('created_at',)
    readonly_fields = ('created_at',)
    inlines = [GeneratedQuestionInline]
    ordering = ('video', 'start_seconds')


@admin.register(SegmentLLMResult)
class SegmentLLMResultAdmin(admin.ModelAdmin):
    list_display = ('id', 'segment', 'has_error', 'created_at', 'updated_at')
    search_fields = ('segment__video__id', 'segment__video__title', 'best_question')
    list_filter = ('has_error', 'created_at', 'updated_at')
    readonly_fields = ('created_at', 'updated_at')
    ordering = ('-updated_at',)


@admin.register(GeneratedQuestion)
class GeneratedQuestionAdmin(admin.ModelAdmin):
    list_display = ('id', 'segment', 'qtype', 'llm_rank')
    search_fields = (
        'segment__video__id',
        'segment__video__title',
        'question',
        'answer',
    )
    list_filter = ('qtype',)
    ordering = ('segment', 'qtype')


@admin.register(SubmittedQuestionSet)
class SubmittedQuestionSetAdmin(admin.ModelAdmin):
    list_display = ('id', 'video', 'status', 'submitted_at')
    search_fields = ('video__id', 'video__title', 'status')
    list_filter = ('status', 'submitted_at')
    readonly_fields = ('submitted_at',)
    ordering = ('-submitted_at',)
