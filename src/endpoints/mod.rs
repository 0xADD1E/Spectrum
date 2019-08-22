mod list_people;
mod list_pronouns;
mod signin;
mod update;

pub use list_people::people;
pub use list_pronouns::pronouns;
pub use signin::signin;
pub use update::update;

pub type ConnPool = actix_web::web::Data<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>;

#[derive(serde::Serialize, Debug)]
pub struct DropdownEntry {
    name: String,
    value: i64,
}
#[derive(serde::Serialize, Debug)]
pub struct ApiResponse {
    success: bool,
    results: Vec<DropdownEntry>,
}
