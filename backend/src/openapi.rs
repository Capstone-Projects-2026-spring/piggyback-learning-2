use utoipa::OpenApi;

use crate::controllers::{answers, frames, kids, openai, parents, questions, tags, videos};

#[derive(OpenApi)]
#[openapi(
    paths(
        answers::analyze_answer,
        answers::get_answers,
        frames::extract_frames,
        kids::get_recommendations,
        kids::get_kid_tags,
        kids::get_video_assignments,
        kids::add_kid_tags,
        kids::create_video_assignment,
        openai::generate_questions,
        parents::get_kids,
        questions::get_questions_by_video,
        tags::get_tags,
        tags::create_tag,
        // // videos
        // videos::download_video,
        // videos::get_video_tags,
    ),
    tags(
        (name = "answers", description = "Answer endpoints"),
        (name = "frames", description = "Frame extraction endpoints"),
        (name = "kids", description = "Kid endpoints"),
        (name = "openai", description = "OpenAI endpoints"),
        (name = "parents", description = "Parent endpoints"),
        (name = "questions", description = "Question endpoints"),
        (name = "tags", description = "Tag endpoints"),
        (name = "videos", description = "Video endpoints"),
    )
)]
pub struct ApiDoc;
