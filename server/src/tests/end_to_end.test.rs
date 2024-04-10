use super::*;
use common::{backend::*, Filter};
#[cfg(test)]
mod end_to_end_test {
    use std::{env, net::TcpStream, time::Duration};

    use log::info;
    use testcontainers::clients;
    use testcontainers_modules::{postgres::Postgres, testcontainers::RunnableImage};
    use tokio::time;

    static INIT: std::sync::Once = std::sync::Once::new();
    fn initialize_logger() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }
    #[tokio::test]
    async fn database_startup_correct() {
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
            .with_env_var(("POSTGRES_DB", "abn"))
            .with_volume((
                "../../database/createTable.sql",
                "/docker-entrypoint-initdb.d/createTable.sql",
            ));

        info!("running docker image");
        let _node = docker.run(postgres_image);
        info!("running main");
        _node.stop();
    }
}
