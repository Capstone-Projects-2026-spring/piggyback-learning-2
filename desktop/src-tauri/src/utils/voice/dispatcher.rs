use super::command_resolver::ResolvedCommand;
use super::onboarding::SharedOnboarding;
use super::session::SharedSession;
use crate::handlers;
use crate::utils::app_handle::emit;
use tauri::AppHandle;

pub async fn dispatch(
    app: AppHandle,
    resolved: ResolvedCommand,
    session: SharedSession,
    onboarding: SharedOnboarding,
) {
    let args = &resolved.args;
    eprintln!("[dispatch] intent={} args={args:?}", resolved.intent);

    if !session.lock().unwrap().is_identified() {
        eprintln!("[dispatch] no user — ignoring '{}'", resolved.intent);
        return;
    }

    match resolved.intent.as_str() {
        // ── kids ──────────────────────────────────────────────────────────────
        "add_kid" => handlers::kids::start_kid_enrollment(&app, args, &session, &onboarding).await,
        "my_tags" => handlers::kids::get_tags(args, &session).await,
        "add_tag" => handlers::kids::add_tags(args, &session).await,
        "my_videos" => handlers::kids::get_video_assignments(args, &session).await,
        "assign_video" => handlers::kids::assign_video(args, &session).await,
        "recommendations" => handlers::kids::get_recommendations(args, &session).await,
        // ── answers ───────────────────────────────────────────────────────────
        "my_answers" => handlers::answers::get_answers_for_session(args, &session).await,
        // ── videos ────────────────────────────────────────────────────────────
        "search" => handlers::videos::search(args).await,
        "watch_video" => emit("orb://watch-video", serde_json::json!({})),
        "download_video" => eprintln!("[dispatch] download_video — handled by frontend"),
        // ── questions ─────────────────────────────────────────────────────────
        "get_questions" => handlers::questions::get_by_video(args, &session).await,
        // ── tags ──────────────────────────────────────────────────────────────
        "all_tags" => handlers::tags::get_all(args, &session).await,
        "create_tag" => handlers::tags::create(args, &session).await,

        other => eprintln!("[dispatch] unhandled intent '{other}'"),
    }
}
