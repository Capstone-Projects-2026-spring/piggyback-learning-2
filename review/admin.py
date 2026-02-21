from django.contrib import admin

from .models import (
    ExpertAnnotation,
    ExpertQuestion,
    FinalAIQuestion,
    FinalQuestionSet,
    FinalSegment,
)


class FinalSegmentInline(admin.TabularInline):
    model = FinalSegment
    extra = 0
    fields = ('segment_index', 'start_seconds', 'end_seconds')
    show_change_link = True


@admin.register(FinalQuestionSet)
class FinalQuestionSetAdmin(admin.ModelAdmin):
    list_display = ('id', 'video', 'saved_at')
    search_fields = ('video__id', 'video__title')
    list_filter = ('saved_at',)
    readonly_fields = ('saved_at',)
    inlines = [FinalSegmentInline]
    ordering = ('-saved_at',)


class FinalAIQuestionInline(admin.TabularInline):
    model = FinalAIQuestion
    extra = 0
    fields = (
        'qtype',
        'llm_ranking',
        'expert_ranking',
        'trashed',
        'question',
        'answer',
    )
    show_change_link = True


@admin.register(FinalSegment)
class FinalSegmentAdmin(admin.ModelAdmin):
    list_display = ('id', 'final_set', 'segment_index', 'start_seconds', 'end_seconds')
    search_fields = (
        'final_set__video__id',
        'final_set__video__title',
    )
    list_filter = ('final_set__saved_at',)
    inlines = [FinalAIQuestionInline]
    ordering = ('final_set', 'start_seconds')


@admin.register(FinalAIQuestion)
class FinalAIQuestionAdmin(admin.ModelAdmin):
    list_display = (
        'id',
        'final_segment',
        'qtype',
        'llm_ranking',
        'expert_ranking',
        'trashed',
    )
    search_fields = (
        'final_segment__final_set__video__id',
        'final_segment__final_set__video__title',
        'question',
        'answer',
    )
    list_filter = ('qtype', 'trashed')
    ordering = ('final_segment', 'qtype')


@admin.register(ExpertAnnotation)
class ExpertAnnotationAdmin(admin.ModelAdmin):
    list_display = (
        'id',
        'video',
        'segment',
        'mode',
        'skipped',
        'question_type',
        'saved_at',
    )
    search_fields = (
        'video__id',
        'video__title',
        'segment__video__id',
        'question',
        'answer',
    )
    list_filter = ('mode', 'skipped', 'question_type')
    readonly_fields = ('saved_at',)
    ordering = ('-saved_at',)


@admin.register(ExpertQuestion)
class ExpertQuestionAdmin(admin.ModelAdmin):
    list_display = (
        'id',
        'video',
        'segment_start',
        'segment_end',
        'timestamp',
        'skipped',
        'question_type',
        'updated_at',
    )
    search_fields = ('video__id', 'video__title', 'question', 'answer', 'skip_reason')
    list_filter = ('skipped', 'question_type')
    readonly_fields = ('updated_at',)
    ordering = ('-updated_at',)
