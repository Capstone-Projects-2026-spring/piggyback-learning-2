use crate::utils::voice::session::SharedSession;

pub async fn analyze_answer(args: &[String], session: &SharedSession) {
    println!("[handler:answers] analyze_answer — args={args:?}");
    // TODO: grab last recorded audio chunk, run similarity + mood detection
}

pub async fn get_answers(args: &[String], session: &SharedSession) {
    println!("[handler:answers] get_answers — args={args:?}");
    // TODO: args[0]=kid_id, args[1]=video_id
}
