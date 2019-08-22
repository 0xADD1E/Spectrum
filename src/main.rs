#![deny(clippy::all)]
use actix_web::{http, middleware, web, App, HttpResponse, HttpServer, Responder};
use log::warn;
use rusqlite::{params, NO_PARAMS};
use serde::{Deserialize, Serialize};

//Static files
mod static_assets;

type ConnPool = web::Data<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>;

#[derive(Serialize, Deserialize, Debug)]
struct DropdownEntry {
    name: String,
    value: i64,
}
#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    success: bool,
    results: Vec<DropdownEntry>,
}

fn index() -> impl Responder {
    HttpResponse::Found()
        .header(http::header::LOCATION, "/static/index.html")
        .finish()
        .into_body()
}

fn people(db: ConnPool) -> Result<impl Responder, failure::Error> {
    let conn = db.get()?;
    let mut nerds = conn.prepare("SELECT people.id,name,subjective,objective,possessive FROM people INNER JOIN pronouns ON people.pronouns = pronouns.rowid")?;
    let nerds: Vec<DropdownEntry> = nerds
        .query_map(NO_PARAMS, |row| {
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
            Ok(DropdownEntry {
                value: id,
                name: format!("{} ({}/{}/{})", &name, &subjective, &objective, &posessive),
            })
        })?
        .filter_map(|obj| match obj {
            Ok(a) => Some(a),
            Err(e) => {
                warn!("Failed to get a person: {:?}", e);
                None
            }
        })
        .collect();
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        results: nerds,
    }))
}
fn pronouns(db: ConnPool) -> Result<impl Responder, failure::Error> {
    let conn = db.get()?;
    let mut pronouns = conn.prepare("SELECT id, subjective,objective,possessive FROM pronouns")?;
    let pronouns: Vec<DropdownEntry> = pronouns
        .query_map(NO_PARAMS, |row| {
            let (id, subjective, objective, posessive): (i64, String, String, String) =
                (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);
            Ok(DropdownEntry {
                value: id,
                name: format!("{}/{}/{}", &subjective, &objective, &posessive),
            })
        })?
        .filter_map(|obj| match obj {
            Ok(a) => Some(a),
            Err(e) => {
                warn!("Failed to get a pronoun set: {:?}", e);
                None
            }
        })
        .collect();
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        results: pronouns,
    }))
}
#[derive(Deserialize)]
struct SigninData {
    pub signin_name: String,
    pub pronouns: Option<i64>,
}
fn signin(form: web::Form<SigninData>, db: ConnPool) -> Result<impl Responder, failure::Error> {
    let conn = db.get()?;
    let person_id = if let Ok(int) = form.signin_name.parse::<i64>() {
        int
    } else {
        let pronouns = form
            .pronouns
            .ok_or_else(|| failure::err_msg("Expected pronouns for signup"))?;
        conn.execute(
            "INSERT INTO people (name, pronouns) VALUES (?1, ?2)",
            params![form.signin_name, pronouns],
        )?;
        conn.last_insert_rowid()
    };
    conn.execute(
        "INSERT INTO signins (person, timestamp) VALUES (?1, DATETIME('now', 'localtime'))",
        params![person_id],
    )?;
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, "/static/index.html#signedin")
        .finish()
        .into_body())
}

#[derive(Deserialize)]
struct UpdateData {
    pub oldname: i64,
    pub newname: Option<String>,
    pub newpronouns: Option<i64>
}
fn update(form: web::Form<UpdateData>, db: ConnPool) -> Result<impl Responder, failure::Error> {
    let conn = db.get()?;
    if let Some(newname) = &form.newname{
        conn.execute("UPDATE people SET name = ?1 WHERE id = ?2", params![newname, form.oldname])?;
    };
    if let Some(newpronouns) = form.newpronouns{
        conn.execute("UPDATE people SET pronouns = ?1 WHERE id = ?2", params![newpronouns, form.oldname])?;
    };
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, "/static/index.html#updated")
        .finish()
        .into_body())
}
fn main() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    let port = port_check::free_local_port_in_range(49152, 65535)
        .ok_or_else(|| failure::err_msg("Failed to obtain port"))?;
    let address = format!("127.0.0.1:{}", port);
    let server = HttpServer::new(move|| {
        let pool = {
            let manager = r2d2_sqlite::SqliteConnectionManager::file("spectrum.db");
            let pool = r2d2::Pool::new(manager).unwrap();
            let swimming = pool.get().unwrap();
            swimming.execute("CREATE TABLE IF NOT EXISTS pronouns (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE, subjective TEXT NOT NULL, objective TEXT NOT NULL, possessive TEXT NOT NULL)", NO_PARAMS).unwrap();
            swimming.execute("CREATE TABLE IF NOT EXISTS people (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE, name TEXT NOT NULL, pronouns INTEGER, FOREIGN KEY (pronouns) REFERENCES pronouns(id))", NO_PARAMS).unwrap();
            swimming.execute("CREATE TABLE IF NOT EXISTS signins (person INTEGER, timestamp TEXT, FOREIGN KEY (person) REFERENCES people(id))", NO_PARAMS).unwrap();
            pool
        };
        let generated = static_assets::generate();
        App::new()
            .wrap(middleware::NormalizePath)
            .data(pool)
            .route("/", web::get().to(index))
            .route("/listPeople", web::get().to(people))
            .route("/listPronouns", web::get().to(pronouns))
            .route("/signin", web::post().to(signin))
            .route("/update", web::post().to(update))
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
