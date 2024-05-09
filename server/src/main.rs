//! Server-Side API crate
#![feature(coverage_attribute)]
#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
mod api;
mod database;
use std::env;

use actix_settings::{ApplySettings as _, BasicSettings};
use actix_web::{dev::Server, web::Data, App, HttpServer};
use api::*;
use log::{info, warn};
use sea_orm::{Database, DatabaseConnection, DbErr, RuntimeErr};
use serde::Deserialize;
use tokio::time::Duration;
static INIT: std::sync::Once = std::sync::Once::new();
fn initialize_logger() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

#[derive(Debug, Deserialize)]
struct DatabaseSettings {
    database_url: String,
}
type AbnSettings = BasicSettings<DatabaseSettings>;

#[coverage(off)]
#[actix_web::main]
async fn main() -> () {
    let server = start_server().await;
    server.await.unwrap();
}

fn load_settings() -> Result<AbnSettings, actix_settings::Error> {
    AbnSettings::parse_toml("Server.toml")
}
// watch for overflows with attempts
async fn connect_to_database_exponential_backoff(
    attempts: u32,
    db_url: String,
) -> Result<DatabaseConnection, DbErr> {
    info!("Attempting to connect to database in {} attempts", attempts);
    let mut attempt: u64 = 1;
    let base: u64 = 2;
    let total_attempts = base.pow(attempts);
    while attempt < total_attempts {
        info!("Attempt {} to connect to database", attempt);
        tokio::time::sleep(Duration::from_secs(attempt)).await;
        match Database::connect(db_url.clone()).await {
            Ok(db) => return Ok(db),
            Err(e) => {
                warn!("Failed to connect to database: {}", e);
                attempt *= 2;
            }
        }
    }
    Err(DbErr::Conn(RuntimeErr::Internal(format!(
        "Failed to connect to database after {} attempts",
        attempts
    ))))
}
#[allow(clippy::needless_return)]
async fn start_server() -> Server {
    env::set_var("RUST_LOG", "info");
    initialize_logger();
    info!("starting server");
    let settings = load_settings().expect("could not load settings");
    info!("loaded settings");
    let db_url = settings.application.database_url.clone();
    info!("connecting to database: {}", db_url.clone());
    let db_connection = connect_to_database_exponential_backoff(4_u32, db_url.clone())
        .await
        .unwrap();
    let db_data: Data<DatabaseConnection> = Data::new(db_connection);
    info!("connected to database");
    info!("creating server");
    let server = HttpServer::new(move || {
        let db_data = db_data.clone();
        App::new()
            .app_data(db_data)
            .service(get_task_request)
            .service(get_task_request)
            .service(get_filter_request)
            .service(create_task_request)
            .service(get_tasks_request)
            .service(update_task_request)
            .service(update_tasks_request)
            .service(delete_task_request)
            .service(delete_tasks_request)
            .service(get_property_request)
            .service(get_properties_request)
    })
    .apply_settings(&settings)
    .system_exit();
    info!("server starting");
    let server_obj = server.run();
    info!("server handle created returning");
    return server_obj;
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
    fn test_logger_no_panic() {
        initialize_logger();
        initialize_logger();
    }
    #[test]
    fn test_main() {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(500));
            std::process::exit(0)
        });
        main();
    }
    #[test]
    fn test_load_settings() {
        load_settings().expect("failed to load settings");
    }
}
#[cfg(test)]
mod server_unit_tests {
    use super::*;
    #[tokio::test]
    async fn test_connect_to_database_exponential_backoff_should_fail_after_31_seconds() {
        let db_url = "postgres://bleh:abn@localhost:5432/abn";
        let start_time = std::time::Instant::now();
        let db_connection =
            connect_to_database_exponential_backoff(4_u32, db_url.to_string()).await;
        assert!(db_connection.is_err());
        assert!(start_time.elapsed().as_secs() > 31);
    }
}
#[cfg(test)]
mod integration_tests {
    use std::{env, net::TcpStream, time::Duration};

    use log::info;
    use testcontainers::clients;
    use testcontainers_modules::{postgres::Postgres, testcontainers::RunnableImage};
    use tokio::time;

    use crate::start_server;
    #[allow(unused_must_use)]
    #[actix_web::test]
    async fn test_database_connection() {
        env::set_var("RUST_LOG", "debug");
        crate::initialize_logger();
        info!("Starting test");
        let docker = clients::Cli::default();
        info!("creating docker image for database");
        info!("{}", std::env::current_dir().unwrap().to_str().unwrap());
        let postgres_image = RunnableImage::from(Postgres::default())
            .with_tag("latest")
            .with_mapped_port((5432, 5432))
            .with_env_var(("POSTGRES_USER", "abn"))
            .with_env_var(("POSTGRES_PASSWORD", "abn"))
            .with_env_var(("POSTGRES_DB", "abn"))
            .with_volume((
                "./database/createTable.sql",
                "/docker-entrypoint-initdb.d/createTable.sql",
            ));
        info!("running docker image");
        let _node = docker.run(postgres_image);
        info!("running main");
        let server = start_server().await;
        let server_handle = server.handle();
        time::sleep(Duration::from_secs(5)).await;
        match TcpStream::connect("127.0.0.1:8080") {
            Ok(_) => {
                info!("connection success");
                assert!(true)
            }
            Err(e) => {
                log::warn!("connection fail");
                assert!(false, "error connecting to server: {}", e)
            }
        }
        info!("stopping docker");
        _node.stop();
        info!("stopping server");
        server_handle.stop(false);
    }
}
