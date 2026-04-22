use super::command_resolver::ResolvedCommand;
use super::onboarding::SharedOnboarding;
use super::session::SharedSession;
use crate::handlers;
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
        eprintln!(
            "[dispatch] no user identified — ignoring '{}'",
            resolved.intent
        );
        return;
    }

    match resolved.intent.as_str() {
        // ── kids ──────────────────────────────────────────────────
        "add_kid" => handlers::kids::start_kid_enrollment(&app, args, &session, &onboarding).await,
        "my_tags" => handlers::kids::get_tags(args, &session).await,
        "add_tag" => handlers::kids::add_tags(args, &session).await,
        "my_videos" => handlers::kids::get_video_assignments(args, &session).await,
        "assign_video" => handlers::kids::assign_video(args, &session).await,
        "recommendations" => handlers::kids::get_recommendations(args, &session).await,
        // ── answers ───────────────────────────────────────────────
        "submit_answer" => handlers::answers::analyze_answer(args, &session).await,
        "my_answers" => handlers::answers::get_answers(args, &session).await,
        // ── parents ───────────────────────────────────────────────
        "my_kids" => handlers::parents::get_kids(args, &session).await,
        // ── videos ────────────────────────────────────────────────
        "download_video" => eprintln!("[dispatch] download_video — handled by frontend"),
        "search" => handlers::videos::search(args, &app).await,
        // ── questions ─────────────────────────────────────────────
        "get_questions" => handlers::questions::get_by_video(args, &session).await,
        "generate_questions" => handlers::questions::generate(args, &session).await,
        // ── tags ──────────────────────────────────────────────────
        "all_tags" => handlers::tags::get_all(args, &session).await,
        "create_tag" => handlers::tags::create(args, &session).await,
        // ── frames ────────────────────────────────────────────────
        "extract_frames" => handlers::frames::extract(args, &session).await,
        // ── misc ──────────────────────────────────────────────────
        "open" => eprintln!("[dispatch] open — args={args:?}"),
        "close" => eprintln!("[dispatch] close — args={args:?}"),
        "play" => eprintln!("[dispatch] play — args={args:?}"),
        "stop" => eprintln!("[dispatch] stop"),
        "volume" => eprintln!("[dispatch] volume — args={args:?}"),
        "help" => eprintln!("[dispatch] help"),
        "chat" => eprintln!("[dispatch] chat — args={args:?}"),
        "wake_only" => eprintln!("[dispatch] wake only"),
        other => eprintln!("[dispatch] unknown intent '{other}'"),
    }
}
