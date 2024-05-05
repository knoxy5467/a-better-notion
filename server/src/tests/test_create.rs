use super::*;
use actix_web::{dev::ServiceResponse, http::StatusCode, test};
use sea_orm::MockDatabase;
use sea_orm::MockExecResult;
use std::vec;

#[actix_web::test]
async fn insert_task_fails_with_bad_request() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres);
    let db_conn = db
        .append_exec_errors([sea_orm::error::DbErr::Query(
            sea_orm::error::RuntimeErr::Internal("test".to_string()),
        )])
        .append_query_errors([sea_orm::error::DbErr::Query(
            sea_orm::error::RuntimeErr::Internal("test".to_string()),
        )])
        .into_connection();
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db_conn))
            .service(create_task_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .set_json(CreateTaskRequest {
            name: "test".to_string(),
            completed: false,
            req_id: 0,
        })
        .uri("/task")
        .to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_web::test]
async fn insert_task_succeeds_with_good_request() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres);
    let db_conn = db
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results([vec![task::Model {
            id: 1,
            title: "test".to_string(),
            completed: false,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .into_connection();
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db_conn))
            .service(create_task_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .set_json(CreateTaskRequest {
            name: "test".to_string(),
            completed: false,
            req_id: 0,
        })
        .uri("/task")
        .to_request();
    let response: CreateTaskResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(
        response,
        CreateTaskResponse {
            task_id: 1,
            req_id: 0
        }
    );
}

#[actix_web::test]
async fn insert_tasks_works() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_exec_results([MockExecResult {
            last_insert_id: 2,
            rows_affected: 1,
        }])
        .append_query_results([vec![task::Model {
            id: 1,
            title: "test".to_string(),
            completed: false,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([vec![task::Model {
            id: 2,
            title: "test2".to_string(),
            completed: false,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .into_connection();
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db))
            .service(create_tasks_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .set_json([
            CreateTaskRequest {
                name: "test".to_string(),
                completed: false,
                req_id: 0,
            },
            CreateTaskRequest {
                name: "test2".to_string(),
                completed: false,
                req_id: 1,
            },
        ])
        .uri("/tasks")
        .to_request();

    let resp: CreateTasksResponse = test::call_and_read_body_json(&app, req).await;
    println!("{:?}", resp);
    assert_eq!(
        resp[0],
        CreateTaskResponse {
            task_id: 1,
            req_id: 0
        }
    );
    assert_eq!(
        resp[1],
        CreateTaskResponse {
            task_id: 2,
            req_id: 1
        }
    );
}
