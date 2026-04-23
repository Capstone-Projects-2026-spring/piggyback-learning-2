use crate::db::init::get_db;
use crate::handlers::questions::generate_questions_for_video;
use crate::utils::app_handle::emit;
use std::collections::HashSet;
use std::fs;
use tokio::process::Command;

pub async fn extract_frames(video_id: &str) -> Result<(), String> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("piggyback")
        .join("downloads")
        .join(video_id);

    let input = data_dir.join(format!("{video_id}.mp4"));
    let output_dir = data_dir.join("extracted_frames");

    if !input.exists() {
        return Err(format!(
            "[frames] video file not found: {}",
            input.display()
        ));
    }

    if output_dir.exists() {
        eprintln!("[frames] already extracted for {video_id} — skipping to questions");
        let vid = video_id.to_string();
        tokio::spawn(async move {
            if let Err(e) = generate_questions_for_video(&vid).await {
                eprintln!("[frames] question generation failed: {e}");
            }
        });
        return Ok(());
    }

    emit(
        "peppa://processing-status",
        serde_json::json!({ "video_id": video_id, "stage": "extracting_frames" }),
    );

    fs::create_dir_all(&output_dir).map_err(|e| format!("[frames] create_dir failed: {e}"))?;

    eprintln!("[frames] detecting scene changes for {video_id}");

    // Pass 1 — detect scene change timestamps via ffmpeg showinfo
    let scene_output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input.to_string_lossy().as_ref())
        .arg("-vf")
        .arg("select='gt(scene\\,0.3)',showinfo")
        .arg("-vsync")
        .arg("vfr")
        .arg("-f")
        .arg("null")
        .arg("-")
        .output()
        .await
        .map_err(|e| format!("[frames] ffmpeg scene pass failed: {e}"))?;

    let stderr = String::from_utf8_lossy(&scene_output.stderr);
    let mut scene_timestamps: HashSet<i32> = HashSet::new();

    for line in stderr.lines() {
        if let Some(pos) = line.find("pts_time:") {
            let rest = &line[pos + 9..];
            let ts_str: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(ts) = ts_str.parse::<f64>() {
                scene_timestamps.insert(ts as i32);
            }
        }
    }

    eprintln!("[frames] scene timestamps: {:?}", scene_timestamps);

    // Pass 2 — extract frames at 1fps
    eprintln!("[frames] starting frame extraction for {video_id}");

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input.to_string_lossy().as_ref())
        .arg("-vf")
        .arg("fps=1")
        .arg(output_dir.join("frame_%04d.jpg").to_string_lossy().as_ref())
        .status()
        .await
        .map_err(|e| format!("[frames] ffmpeg extract failed: {e}"))?;

    if !status.success() {
        return Err("[frames] ffmpeg exited with non-zero status".to_string());
    }

    let mut files: Vec<_> = fs::read_dir(&output_dir)
        .map_err(|e| format!("[frames] read_dir failed: {e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    files.sort_by_key(|f| f.path());

    if files.is_empty() {
        return Err("[frames] ffmpeg produced no frames".to_string());
    }

    let pool = get_db();

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("[frames] tx begin failed: {e}"))?;

    for (idx, entry) in files.iter().enumerate() {
        let path = entry.path();
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let timestamp_seconds = idx as i32;
        let timestamp_formatted = format!(
            "{:02}:{:02}",
            timestamp_seconds / 60,
            timestamp_seconds % 60
        );
        let is_keyframe = if scene_timestamps.contains(&timestamp_seconds) {
            1i32
        } else {
            0i32
        };

        sqlx::query(
            "INSERT INTO frames
             (video_id, frame_number, timestamp_seconds, timestamp_formatted,
              filename, file_path, is_keyframe)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(video_id)
        .bind(idx as i32)
        .bind(timestamp_seconds)
        .bind(&timestamp_formatted)
        .bind(&filename)
        .bind(path.to_string_lossy().as_ref())
        .bind(is_keyframe)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("[frames] insert failed at frame {idx}: {e}"))?;
    }

    tx.commit()
        .await
        .map_err(|e| format!("[frames] tx commit failed: {e}"))?;

    eprintln!(
        "[frames] extracted and saved {} frames for {video_id} ({} keyframes)",
        files.len(),
        scene_timestamps.len()
    );

    let vid = video_id.to_string();
    tokio::spawn(async move {
        if let Err(e) = generate_questions_for_video(&vid).await {
            eprintln!("[frames] question generation failed: {e}");
        }
    });

    Ok(())
}
