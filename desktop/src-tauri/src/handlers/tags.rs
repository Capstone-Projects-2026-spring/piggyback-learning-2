use crate::{db::init::get_db, utils::voice::session::SharedSession};

pub async fn get_all(_args: &[String], _session: &SharedSession) {
    let pool = get_db();
    match sqlx::query_as::<_, (i64, String)>("SELECT id, name FROM tags ORDER BY name")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let tags: Vec<String> = rows
                .iter()
                .map(|(id, name)| format!("{id}:{name}"))
                .collect();
            eprintln!("[handler:tags] all_tags → {:?}", tags);
        }
        Err(e) => eprintln!("[handler:tags] get_all failed: {e}"),
    }
}

pub async fn create(args: &[String], _session: &SharedSession) {
    let Some(name) = args.first() else {
        eprintln!("[handler:tags] create — no name provided");
        return;
    };
    match get_or_create_tag(name).await {
        Ok(id) => eprintln!("[handler:tags] create → id={id} name={name}"),
        Err(e) => eprintln!("[handler:tags] create failed: {e}"),
    }
}

/// Returns the id of an existing tag with this name, or inserts and returns the new one.
pub async fn get_or_create_tag(name: &str) -> Result<i64, String> {
    let pool = get_db();
    let name = name.trim().to_lowercase();

    // Try existing first
    let existing = sqlx::query_as::<_, (i64,)>("SELECT id FROM tags WHERE name = ?")
        .bind(&name)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("[tags] select failed: {e}"))?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    let row = sqlx::query("INSERT INTO tags (name) VALUES (?) RETURNING id")
        .bind(&name)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("[tags] insert failed: {e}"))?;

    let id: i64 = sqlx::Row::try_get(&row, "id").map_err(|e| format!("[tags] id fetch: {e}"))?;

    eprintln!("[tags] created tag id={id} name={name}");
    Ok(id)
}
