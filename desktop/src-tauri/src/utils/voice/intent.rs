use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    Search,
    AddKid,
    MyAnswers,
    AddTag,
    MyVideos,
    AssignVideo,
    WatchVideo,
    Recommendations,
    DownloadVideo,
    // Pipeline-internal, never dispatched
    WakeOnly,
    Unhandled,
}

impl Intent {
    pub fn from_str(s: &str) -> Self {
        match s {
            "search" => Self::Search,
            "add_kid" => Self::AddKid,
            "my_answers" => Self::MyAnswers,
            "add_tag" => Self::AddTag,
            "my_videos" => Self::MyVideos,
            "assign_video" => Self::AssignVideo,
            "watch_video" => Self::WatchVideo,
            "recommendations" => Self::Recommendations,
            "download_video" => Self::DownloadVideo,
            _ => Self::Unhandled,
        }
    }
}
