use crate::utils::voice::session::SharedSession;

pub async fn get_by_video(args: &[String], session: &SharedSession) {
    println!("[handler:questions] get_by_video — args={args:?}");
    // TODO: args[0]=video_id
}

pub async fn generate(args: &[String], session: &SharedSession) {
    println!("[handler:questions] generate — args={args:?}");
    // TODO: args[0]=video_id, args[1]=start_secs, args[2]=end_secs
}
