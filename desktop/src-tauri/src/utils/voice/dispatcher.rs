use super::command_resolver::ResolvedCommand;
use crate::handlers;
use crate::utils::voice::session::SharedSession;

pub async fn dispatch(resolved: ResolvedCommand, session: SharedSession) {
    let args = &resolved.args;
    eprintln!("[dispatch] intent={} args={args:?}", resolved.intent);

    // Guard — don't run commands if no user is identified yet
    if !session.lock().unwrap().is_identified() {
        eprintln!(
            "[dispatch] no user identified — ignoring intent '{}'",
            resolved.intent
        );
        return;
    }

    match resolved.intent.as_str() {
        "open" => eprintln!("[dispatch] open — args={args:?}"),
        "close" => eprintln!("[dispatch] close — args={args:?}"),
        "play" => eprintln!("[dispatch] play — args={args:?}"),
        "stop" => eprintln!("[dispatch] stop"),
        "search" => eprintln!("[dispatch] search — args={args:?}"),
        "volume" => eprintln!("[dispatch] volume — args={args:?}"),
        "help" => eprintln!("[dispatch] help"),
        "chat" => eprintln!("[dispatch] chat — args={args:?}"),
        "wake_only" => eprintln!("[dispatch] wake only"),

        // ── answers ───────────────────────────────────────────────
        "submit_answer" => handlers::answers::analyze_answer(args, &session).await,
        "my_answers" => handlers::answers::get_answers(args, &session).await,

        // ── kids ──────────────────────────────────────────────────
        "my_tags" => handlers::kids::get_tags(args, &session).await,
        "add_tag" => handlers::kids::add_tags(args, &session).await,
        "my_videos" => handlers::kids::get_video_assignments(args, &session).await,
        "assign_video" => handlers::kids::assign_video(args, &session).await,
        "recommendations" => handlers::kids::get_recommendations(args, &session).await,

        // ── parents ───────────────────────────────────────────────
        "my_kids" => handlers::parents::get_kids(args, &session).await,

        // ── videos ────────────────────────────────────────────────
        "download_video" => handlers::videos::download(args, &session).await,
        "video_tags" => handlers::videos::get_tags(args, &session).await,

        // ── questions ─────────────────────────────────────────────
        "get_questions" => handlers::questions::get_by_video(args, &session).await,
        "generate_questions" => handlers::questions::generate(args, &session).await,

        // ── tags ──────────────────────────────────────────────────
        "all_tags" => handlers::tags::get_all(args, &session).await,
        "create_tag" => handlers::tags::create(args, &session).await,

        // ── frames ────────────────────────────────────────────────
        "extract_frames" => handlers::frames::extract(args, &session).await,

        other => eprintln!("[dispatch] unknown intent '{other}'"),
    }
}
