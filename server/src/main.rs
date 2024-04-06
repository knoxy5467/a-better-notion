//! Server-Side API crate
#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
mod api;
mod database;
use actix_web::{web::Data, App, HttpServer};
use api::*;
use sea_orm::{Database, DatabaseConnection};
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server");

    let db =
        Database::connect("postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask")
            .await
            .unwrap();
    let db_data: Data<DatabaseConnection> = Data::new(db);
    HttpServer::new(move || {
        let db_data = db_data.clone();
        App::new()
            .app_data(db_data)
            .service(get_task_request)
            .service(get_tasks_request)
            .service(get_filter_request)
            .service(create_task_request)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
#[path = "./tests/test_filter.rs"]
mod test_filter;
#[cfg(test)]
#[path = "./tests/test_tasks.rs"]
mod test_tasks;

#[cfg(test)]
mod test_main {
    use super::*;

    #[test]
    fn test_main() {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(500));
            std::process::exit(0)
        });
        main().unwrap();
    }
}
