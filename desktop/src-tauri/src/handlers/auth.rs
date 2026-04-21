use crate::utils::voice::session::SharedSession;

pub async fn login(args: &[String], session: &SharedSession) {
    println!("[handler:auth] login — args={args:?}");
    // TODO: args[0]=username, args[1]=password, args[2]=role
}

pub async fn signup(args: &[String], session: &SharedSession) {
    println!("[handler:auth] signup — args={args:?}");
    // TODO: args[0]=name, args[1]=username, args[2]=password, args[3]=role
}
