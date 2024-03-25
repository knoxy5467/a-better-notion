//! Server-Side API crate

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
mod api;
use actix_web::{App, HttpServer};
use api::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server");

    HttpServer::new(|| {
        App::new()
            .service(get_task_request)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_it_test_main() {
        main();
        assert!(true);
    }
}
