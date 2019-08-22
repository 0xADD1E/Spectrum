#![deny(clippy::all)]
mod endpoints;
mod static_assets;

fn index() -> impl actix_web::Responder {
    actix_web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, "/static/index.html")
        .finish()
        .into_body()
}

fn main() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    let port = port_check::free_local_port_in_range(49152, 65535)
        .ok_or_else(|| failure::err_msg("Failed to obtain port"))?;
    let address = format!("127.0.0.1:{}", port);
    let server = actix_web::HttpServer::new(move|| {
        use actix_web::web;
        let pool = {
            use rusqlite::NO_PARAMS;
            let manager = r2d2_sqlite::SqliteConnectionManager::file("spectrum.db");
            let pool = r2d2::Pool::new(manager).unwrap();
            let swimming = pool.get().unwrap();
            swimming.execute("CREATE TABLE IF NOT EXISTS pronouns (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE, subjective TEXT NOT NULL, objective TEXT NOT NULL, possessive TEXT NOT NULL)", NO_PARAMS).unwrap();
            swimming.execute("CREATE TABLE IF NOT EXISTS people (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE, name TEXT NOT NULL, pronouns INTEGER, FOREIGN KEY (pronouns) REFERENCES pronouns(id))", NO_PARAMS).unwrap();
            swimming.execute("CREATE TABLE IF NOT EXISTS signins (person INTEGER, timestamp TEXT, FOREIGN KEY (person) REFERENCES people(id))", NO_PARAMS).unwrap();
            pool
        };
        let generated = static_assets::generate();
        actix_web::App::new()
            .wrap(actix_web::middleware::NormalizePath)
            .data(pool)
            .route("/", web::get().to(index))
            .route("/listPeople", web::get().to(endpoints::people))
            .route("/listPronouns", web::get().to(endpoints::pronouns))
            .route("/signin", web::post().to(endpoints::signin))
            .route("/update", web::post().to(endpoints::update))
            .service(actix_web_static_files::ResourceFiles::new(
                "/static", generated,
            ))
    })
    .bind(&address)
    .unwrap();

    let address = format!("http://{}", address);
    if webbrowser::open(&address).is_ok() {
        server.run()?;
        Ok(())
    } else {
        Err(failure::err_msg("Couldn't open web browser :("))
    }
}
