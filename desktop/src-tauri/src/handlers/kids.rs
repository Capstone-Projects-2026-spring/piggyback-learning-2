use crate::utils::voice::session::SharedSession;

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
    // TODO: args[0]=kid_id, args[1]=video_id
}

pub async fn get_recommendations(args: &[String], session: &SharedSession) {
    println!("[handler:kids] get_recommendations — args={args:?}");
    // TODO: args[0]=kid_id
}
