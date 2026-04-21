use crate::utils::voice::session::SharedSession;

pub async fn get_kids(args: &[String], session: &SharedSession) {
    println!("[handler:parents] get_kids — args={args:?}");
    // TODO: args[0]=parent_id
}
