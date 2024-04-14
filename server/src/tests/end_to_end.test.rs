use super::*;
use common::{backend::*, Filter};
use std::{env, net::TcpStream, time::Duration};

use log::info;
use testcontainers::{clients, Container};
use testcontainers_modules::{postgres::Postgres, testcontainers::RunnableImage};
use tokio::time;

static INIT: std::sync::Once = std::sync::Once::new();
fn initialize_logger() {
    INIT.call_once(|| {
        env_logger::init();
    });
}
/*
async fn start_test_database() -> Container<'static, Postgres> {
    let docker = clients::Cli::default();
    info!("creating docker image for database");
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
    let node = docker.run(postgres_image);
    return node;
}
*/
#[tokio::test]
async fn database_startup_correct() {
    env::set_var("RUST_LOG", "info");
    crate::initialize_logger();
    info!("Starting test");
    info!("running main");
    // let node = start_test_database().await;
}

async fn start_server() {
    let server = crate::start_server().await;
    todo!("use reqest to test server");
}
