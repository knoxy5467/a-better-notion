use log::info;
use testcontainers::{clients, Container, RunnableImage};
use testcontainers_modules::postgres::Postgres;

use std::sync::OnceLock;
pub static DB: OnceLock<Container<'static, Postgres>> = OnceLock::new();
pub fn setup_db() {
    DB.get_or_init(|| {
        let docker = Box::leak(Box::new(clients::Cli::default()));
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
        docker.run(postgres_image)
    });
}
