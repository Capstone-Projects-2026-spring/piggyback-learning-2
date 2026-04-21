use crate::utils::voice::{
    onboarding::{self, OnboardingFlow, SharedOnboarding},
    session::SharedSession,
};
use tauri::AppHandle;

pub async fn get_tags(args: &[String], session: &SharedSession) {
    println!("[handler:kids] get_tags — args={args:?}");
}

pub async fn add_tags(args: &[String], session: &SharedSession) {
    println!("[handler:kids] add_tags — args={args:?}");
}

pub async fn get_video_assignments(args: &[String], session: &SharedSession) {
    println!("[handler:kids] get_video_assignments — args={args:?}");
}

pub async fn assign_video(args: &[String], session: &SharedSession) {
    println!("[handler:kids] assign_video — args={args:?}");
}

pub async fn get_recommendations(args: &[String], session: &SharedSession) {
    println!("[handler:kids] get_recommendations — args={args:?}");
}

pub async fn start_kid_enrollment(
    app: &AppHandle,
    args: &[String],
    session: &SharedSession,
    onboarding: &SharedOnboarding,
) {
    let role = session.lock().unwrap().role.clone();

    match role.as_deref() {
        Some("parent") => {
            eprintln!("[handler:kids] starting kid enrollment");
            onboarding::start(app, onboarding, OnboardingFlow::Kid);
        }
        Some(r) => eprintln!("[handler:kids] add_kid — role={r} is not a parent, ignoring"),
        None => eprintln!("[handler:kids] add_kid — no user identified, ignoring"),
    }
}
