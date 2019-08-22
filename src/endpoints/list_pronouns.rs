pub fn pronouns(db: super::ConnPool) -> Result<impl actix_web::Responder, failure::Error> {
    let conn = db.get()?;
    let mut pronouns = conn.prepare("SELECT id, subjective,objective,possessive FROM pronouns")?;
    let pronouns: Vec<super::DropdownEntry> = pronouns
        .query_map(rusqlite::NO_PARAMS, |row| {
            let (id, subjective, objective, posessive): (i64, String, String, String) =
                (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);
            Ok(super::DropdownEntry {
                value: id,
                name: format!("{}/{}/{}", &subjective, &objective, &posessive),
            })
        })?
        .filter_map(|obj| match obj {
            Ok(a) => Some(a),
            Err(e) => {
                log::warn!("Failed to get a pronoun set: {:?}", e);
                None
            }
        })
        .collect();
    Ok(actix_web::HttpResponse::Ok().json(super::ApiResponse {
        success: true,
        results: pronouns,
    }))
}
