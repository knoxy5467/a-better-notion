//! Server-Side API crate

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
mod api;
mod database;
use actix_web::{App, HttpServer};
use api::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server");

    HttpServer::new(|| {
        App::new()
            .service(get_task_request)
            .service(get_tasks_request)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::backend::{ReadTaskShortRequest, ReadTaskShortResponse};

    #[actix_web::test]
    async fn task_request() {
        use actix_web::test;
        let app = test::init_service(App::new().service(get_task_request)).await;
        let req = test::TestRequest::default()
            .set_json(ReadTaskShortRequest { task_id: 1 })
            .uri("/task")
            .to_request();
        let resp: ReadTaskShortResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.task_id, 1);
    }
}
