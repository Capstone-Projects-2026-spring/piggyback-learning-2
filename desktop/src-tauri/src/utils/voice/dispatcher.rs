use super::command_resolver::ResolvedCommand;
use super::intent::Intent;
use super::onboarding::SharedOnboarding;
use super::session::SharedSession;
use crate::handlers;
use crate::utils::app_handle::emit;

pub async fn dispatch(
    resolved: ResolvedCommand,
    session: SharedSession,
    onboarding: SharedOnboarding,
) {
    let args = &resolved.args;
    eprintln!("[dispatch] intent={:?} args={args:?}", resolved.intent);

    if !session.lock().unwrap().is_identified() {
        eprintln!("[dispatch] no user — ignoring {:?}", resolved.intent);
        return;
    }

    match resolved.intent {
        Intent::AddKid => handlers::kids::start_kid_enrollment(args, &session, &onboarding).await,
        Intent::AddTag => handlers::kids::add_tags(args, &session).await,
        Intent::MyVideos => handlers::kids::get_video_assignments(args, &session).await,
        Intent::AssignVideo => handlers::kids::assign_video(args, &session).await,
        Intent::Recommendations => handlers::kids::get_recommendations(args, &session).await,
        Intent::MyAnswers => handlers::answers::get_answers_for_session(args, &session).await,
        Intent::Search => handlers::videos::search(args).await,
        Intent::WatchVideo => emit("orb://watch-video", serde_json::json!({})),
        Intent::DownloadVideo => eprintln!("[dispatch] download_video — handled by frontend"),
        // These never reach dispatch rather filtered out in capture.rs
        Intent::WakeOnly | Intent::Unhandled => {}
    }
}
