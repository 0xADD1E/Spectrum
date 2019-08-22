pub fn people(db: super::ConnPool) -> Result<impl actix_web::Responder, failure::Error> {
    let conn = db.get()?;
    let mut nerds = conn.prepare("SELECT people.id,name,subjective,objective,possessive FROM people INNER JOIN pronouns ON people.pronouns = pronouns.rowid")?;
    let nerds: Vec<super::DropdownEntry> = nerds
        .query_map(rusqlite::NO_PARAMS, |row| {
            let (id, name, subjective, objective, posessive): (
                i64,
                String,
                String,
                String,
                String,
            ) = (
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            );
            Ok(super::DropdownEntry {
                value: id,
                name: format!("{} ({}/{}/{})", &name, &subjective, &objective, &posessive),
            })
        })?
        .filter_map(|obj| match obj {
            Ok(a) => Some(a),
            Err(e) => {
                log::warn!("Failed to get a person: {:?}", e);
                None
            }
        })
        .collect();
    Ok(actix_web::HttpResponse::Ok().json(super::ApiResponse {
        success: true,
        results: nerds,
    }))
}
