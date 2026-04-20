pub async fn download(args: &[String]) {
    println!("[handler:videos] download — args={args:?}");
    // TODO: args[0]=video_id, shell out to yt-dlp
}

pub async fn get_tags(args: &[String]) {
    println!("[handler:videos] get_tags — args={args:?}");
    // TODO: args[0]=video_id
}

pub async fn add_tags(args: &[String]) {
    println!("[handler:videos] add_tags — args={args:?}");
    // TODO: args[0]=video_id, args[1..]=tag_ids
}
