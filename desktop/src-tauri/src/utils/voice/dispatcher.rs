use crate::handlers;

use super::command_resolver::ResolvedCommand;

pub async fn dispatch(resolved: ResolvedCommand) {
    let args = &resolved.args;
    println!("[dispatch] intent={} args={args:?}", resolved.intent);

    match resolved.intent.as_str() {
        // ── existing ──────────────────────────────────────────────
        "open" => println!("[dispatch] open — args={args:?}"),
        "close" => println!("[dispatch] close — args={args:?}"),
        "play" => println!("[dispatch] play — args={args:?}"),
        "stop" => println!("[dispatch] stop"),
        "search" => println!("[dispatch] search — args={args:?}"),
        "volume" => println!("[dispatch] volume — args={args:?}"),
        "help" => println!("[dispatch] help"),
        "chat" => println!("[dispatch] chat — args={args:?}"),
        "wake_only" => println!("[dispatch] wake only, no command"),

        // ── auth ──────────────────────────────────────────────────
        "login" => handlers::auth::login(args).await,
        "signup" => handlers::auth::signup(args).await,

        // ── answers ───────────────────────────────────────────────
        "submit_answer" => handlers::answers::analyze_answer(args).await,
        "my_answers" => handlers::answers::get_answers(args).await,

        // ── kids ──────────────────────────────────────────────────
        "my_tags" => handlers::kids::get_tags(args).await,
        "add_tag" => handlers::kids::add_tags(args).await,
        "my_videos" => handlers::kids::get_video_assignments(args).await,
        "assign_video" => handlers::kids::assign_video(args).await,
        "recommendations" => handlers::kids::get_recommendations(args).await,

        // ── parents ───────────────────────────────────────────────
        "my_kids" => handlers::parents::get_kids(args).await,

        // ── videos ────────────────────────────────────────────────
        "download_video" => handlers::videos::download(args).await,
        "video_tags" => handlers::videos::get_tags(args).await,

        // ── questions ─────────────────────────────────────────────
        "get_questions" => handlers::questions::get_by_video(args).await,
        "generate_questions" => handlers::questions::generate(args).await,

        // ── tags ──────────────────────────────────────────────────
        "all_tags" => handlers::tags::get_all(args).await,
        "create_tag" => handlers::tags::create(args).await,

        // ── frames ────────────────────────────────────────────────
        "extract_frames" => handlers::frames::extract(args).await,

        other => println!("[dispatch] unknown intent '{other}', ignoring"),
    }
}
