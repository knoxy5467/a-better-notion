//! Server-Side API crate
#![feature(coverage_attribute)]
#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
mod api;
mod database;
use actix_web::{
    dev::{Server, ServerHandle},
    web::Data,
    App, HttpServer,
};
use api::*;
use log::{info, warn};
use sea_orm::{Database, DatabaseConnection};
static INIT: std::sync::Once = std::sync::Once::new();
fn initialize_logger() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

#[coverage(off)]
#[actix_web::main]
async fn main() -> () {
    let (server_handle, _server) = start_server().await;
    tokio::signal::ctrl_c().await.unwrap();
    server_handle.stop(true).await;
}
#[allow(clippy::needless_return)]
async fn start_server() -> (ServerHandle, Server) {
    initialize_logger();
    let db =
        Database::connect("postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask")
            .await
            .unwrap();
    let db_data: Data<DatabaseConnection> = Data::new(db);
    let server = HttpServer::new(move || {
        let db_data = db_data.clone();
        App::new()
            .app_data(db_data)
            .service(get_task_request)
            .service(get_tasks_request)
            .service(get_filter_request)
            .service(create_task_request)
    })
    .bind(("127.0.0.1", 8080))
    .unwrap();
    info!("server starting");
    let server_obj = server.run();
    info!("server started, creating handle");
    let server_handle = server_obj.handle();
    info!("server handle created returning");
    return (server_handle, server_obj);
}

#[cfg(test)]
#[path = "./tests/test_filter.rs"]
mod test_filter;
#[cfg(test)]
#[path = "./tests/test_tasks.rs"]
mod test_tasks;
#[cfg(test)]
#[path = "./tests/test_update.rs"]
mod test_update;

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
}

#[cfg(test)]
mod integration_tests {
    use std::{env, net::TcpStream, time::Duration};

    use log::info;
    use testcontainers::clients;
    use testcontainers_modules::{postgres::Postgres, testcontainers::RunnableImage};
    use tokio::time;

    use crate::start_server;
    #[actix_web::test]
    async fn test_database_connection() {
        env::set_var("RUST_LOG", "info");
        crate::initialize_logger();
        info!("Starting test");
        let docker = clients::Cli::default();
        info!("creating docker image for database");
        let postgres_image = RunnableImage::from(Postgres::default())
            .with_tag("latest")
            .with_mapped_port((5432, 5432))
            .with_env_var(("POSTGRES_USER", "abn"))
            .with_env_var(("POSTGRES_PASSWORD", "abn"))
            .with_env_var(("POSTGRES_DB", "abn"));
        info!("running docker image");
        let _node = docker.run(postgres_image);
        info!("running main");
        info!("current system {:?}", actix_web::rt::System::current());
        let (server_handle, server_obj) = start_server().await;

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
        let _unused_future = server_obj.handle().stop(false);
        info!("stopping server");
        let _res = server_handle.stop(false);
    }
}
