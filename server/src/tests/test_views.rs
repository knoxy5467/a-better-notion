use super::*;
use actix_web::test;
use common::Filter;
use sea_orm::MockDatabase;
use sea_orm::MockExecResult;
use std::vec;

macro_rules! mk_app {
    ($req:ident, $app:ident, $db:expr, $func:expr, $method:expr, $uri:expr, $json:expr) => {
        let $app = test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new($db))
                .service($func),
        )
        .await;
        let $req = test::TestRequest::default()
            .method($method)
            .set_json($json)
            .uri($uri)
            .to_request();
    };
}

#[actix_web::test]
async fn test_create_view() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();
    mk_app!(
        req,
        app,
        db,
        create_view_request,
        actix_web::http::Method::POST,
        "/view",
        CreateViewRequest {
            name: "".to_string(),
            props: vec![],
            filter: Filter::None,
            req_id: 0,
        }
    );

    test::call_service(&app, req).await;
}

#[actix_web::test]
async fn test_update_view() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[view::Model {
            id: 0,
            name: "idk".to_string(),
            properties: vec![],
            filter: "{}".to_string(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();
    mk_app!(
        req,
        app,
        db,
        update_view_request,
        actix_web::http::Method::PUT,
        "/view",
        UpdateViewRequest {
            view: ViewData {
                view_id: 0,
                name: "heyo".to_string(),
                filter: Filter::None,
                props: vec![]
            },
            req_id: 0,
        }
    );

    test::call_service(&app, req).await;
}

#[actix_web::test]
async fn test_get_views() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[view::Model {
            id: 0,
            name: "idk".to_string(),
            properties: vec![],
            filter: serde_json::to_string(&Filter::None).unwrap(),
        }]])
        .into_connection();

    mk_app!(
        req,
        app,
        db,
        get_views_request,
        actix_web::http::Method::GET,
        "/views",
        0
    );

    test::call_service(&app, req).await;
}

#[actix_web::test]
async fn test_delete_view() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[view::Model {
            id: 0,
            name: "idk".to_string(),
            properties: vec![],
            filter: "{}".to_string(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    mk_app!(
        req,
        app,
        db,
        delete_view_request,
        actix_web::http::Method::DELETE,
        "/view",
        0
    );

    test::call_service(&app, req).await;
}
