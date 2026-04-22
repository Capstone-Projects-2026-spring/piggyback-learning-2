use crate::db::init::get_db;
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
        eprintln!("[frames] already extracted for {video_id} — skipping");
        return Ok(());
    }

    fs::create_dir_all(&output_dir).map_err(|e| format!("[frames] create_dir failed: {e}"))?;

    eprintln!("[frames] starting ffmpeg for {video_id}");

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input.to_string_lossy().as_ref())
        .arg("-vf")
        .arg("fps=1")
        .arg(output_dir.join("frame_%04d.jpg").to_string_lossy().as_ref())
        .status()
        .await
        .map_err(|e| format!("[frames] ffmpeg spawn failed: {e}"))?;

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
    let fps = 1.0_f64;

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
        let timestamp_seconds = (idx as f64 / fps) as i32;
        let timestamp_formatted = format!(
            "{:02}:{:02}",
            timestamp_seconds / 60,
            timestamp_seconds % 60
        );

        sqlx::query(
            "INSERT OR IGNORE INTO frames
             (video_id, frame_number, timestamp_seconds, timestamp_formatted,
              filename, file_path, subtitle_text, is_keyframe)
             VALUES (?, ?, ?, ?, ?, ?, NULL, 0)",
        )
        .bind(video_id)
        .bind(idx as i32)
        .bind(timestamp_seconds)
        .bind(&timestamp_formatted)
        .bind(&filename)
        .bind(path.to_string_lossy().as_ref())
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("[frames] insert failed at frame {idx}: {e}"))?;
    }

    tx.commit()
        .await
        .map_err(|e| format!("[frames] tx commit failed: {e}"))?;

    eprintln!(
        "[frames] extracted and saved {} frames for {video_id}",
        files.len()
    );
    Ok(())
}
