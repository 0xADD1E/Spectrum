#[derive(serde::Deserialize)]
pub struct UpdateData {
    pub oldname: i64,
    pub newname: Option<String>,
    pub newpronouns: Option<i64>,
}
pub fn update(
    form: actix_web::web::Form<UpdateData>,
    db: super::ConnPool,
) -> Result<impl actix_web::Responder, failure::Error> {
    let conn = db.get()?;
    if let Some(newname) = &form.newname {
        conn.execute(
            "UPDATE people SET name = ?1 WHERE id = ?2",
            rusqlite::params![newname, form.oldname],
        )?;
    };
    if let Some(newpronouns) = form.newpronouns {
        conn.execute(
            "UPDATE people SET pronouns = ?1 WHERE id = ?2",
            rusqlite::params![newpronouns, form.oldname],
        )?;
    };
    Ok(actix_web::HttpResponse::Found()
        .header(
            actix_web::http::header::LOCATION,
            "/static/index.html#updated",
        )
        .finish()
        .into_body())
}
