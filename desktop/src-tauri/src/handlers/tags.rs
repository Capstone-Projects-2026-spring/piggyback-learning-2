use crate::utils::voice::session::SharedSession;

pub async fn get_all(_args: &[String], session: &SharedSession) {
    println!("[handler:tags] get_all");
}

pub async fn create(args: &[String], session: &SharedSession) {
    println!("[handler:tags] create — args={args:?}");
    // TODO: args[0..]=tag name words
}
