use utoipa::OpenApi;

use crate::controllers::{answers, frames, kids, openai, parents, questions, tags, videos};

#[derive(OpenApi)]
#[openapi(
    paths(
        // answers
        answers::analyze_answer,
        answers::get_answers,
        // frames
        // frames::extract_frames,
        // // kids
        // kids::get_recommendations,
        // kids::get_tags,
        // kids::get_videos_assigned,
        // // openai
        // openai::get_openai,
        // // parents
        // parents::get_kids,
        // // questions
        // questions::get_questions,
        // // tags
        // tags::get_tags,
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
