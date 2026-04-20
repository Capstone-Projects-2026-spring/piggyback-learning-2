pub async fn get_all(_args: &[String]) {
    println!("[handler:tags] get_all");
}

pub async fn create(args: &[String]) {
    println!("[handler:tags] create — args={args:?}");
    // TODO: args[0..]=tag name words
}
