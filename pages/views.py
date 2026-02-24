from django.http import HttpRequest
from django.shortcuts import render

from videos.models import Video


def home(request: HttpRequest):
    return render(request, 'home.html')


def children(request: HttpRequest):
    return render(request, 'children.html')


def admin_ui(request: HttpRequest):
    return render(request, 'admin.html')


def expert_preview(request: HttpRequest):
    """
    FastAPI's /expert-preview renders expert_preview.html with a big context
    (question files, selected video/file, annotations, etc).
    In our DB version, we can still render the template and supply what it needs.

    Minimal viable context (won't crash templates):
      - selected_video_id
      - mode
      - video_url (if you still serve /downloads/<...>)
      - segments_for_js (empty list until you wire it)
    """
    mode = request.GET.get('mode', 'review')
    video_id = request.GET.get('video')

    video_url = None
    if video_id:
        v = Video.objects.filter(id=video_id).first()
        if v and v.local_video_path:
            video_url = v.local_video_path

    context = {
        'mode': mode,
        'selected_video_id': video_id,
        'video_url': video_url,
        'segments_for_js': [],  # later you can fill from Segment + GeneratedQuestion or FinalQuestionSet
        'question_files': [],  # legacy, if template lists files
        'selection_error': None,
        'existing_annotations': [],
        'existing_annotations_map': {},
        'selected_json_pretty': None,
        'annotations_rel_path': None,
        'question_file_url': None,
        'selected_file_rel': None,
        'selected_file_name': None,
        'segments': [],
        'question_type_options': [
            {'value': 'character', 'label': 'Character'},
            {'value': 'setting', 'label': 'Setting'},
            {'value': 'feeling', 'label': 'Feeling'},
            {'value': 'action', 'label': 'Action'},
            {'value': 'causal', 'label': 'Causal'},
            {'value': 'outcome', 'label': 'Outcome'},
            {'value': 'prediction', 'label': 'Prediction'},
        ],
    }
    return render(request, 'expert_preview.html', context)
