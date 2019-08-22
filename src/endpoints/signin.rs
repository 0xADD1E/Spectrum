#[derive(serde::Deserialize)]
pub struct SigninData {
    pub signin_name: String,
    pub pronouns: Option<i64>,
}
pub fn signin(
    form: actix_web::web::Form<SigninData>,
    db: super::ConnPool,
) -> Result<impl actix_web::Responder, failure::Error> {
    let conn = db.get()?;
    let person_id = if let Ok(int) = form.signin_name.parse::<i64>() {
        int
    } else {
        let pronouns = form
            .pronouns
            .ok_or_else(|| failure::err_msg("Expected pronouns for signup"))?;
        conn.execute(
            "INSERT INTO people (name, pronouns) VALUES (?1, ?2)",
            rusqlite::params![form.signin_name, pronouns],
        )?;
        conn.last_insert_rowid()
    };
    conn.execute(
        "INSERT INTO signins (person, timestamp) VALUES (?1, DATETIME('now', 'localtime'))",
        rusqlite::params![person_id],
    )?;
    Ok(actix_web::HttpResponse::Found()
        .header(
            actix_web::http::header::LOCATION,
            "/static/index.html#signedin",
        )
        .finish()
        .into_body())
}
