use crate::utils::voice::session::SharedSession;

pub async fn extract(args: &[String], session: &SharedSession) {
    println!("[handler:frames] extract — args={args:?}");
    // TODO: args[0]=video_id, shell out to ffmpeg
}
