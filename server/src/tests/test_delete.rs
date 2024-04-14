use super::*;
use actix_web::test;
use sea_orm::{MockDatabase, MockExecResult};

#[actix_web::test]
async fn test_delete() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let res = delete_task(&db, &DeleteTaskRequest { task_id: 1 }).await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn test_delete_bad_id() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([vec![] as Vec<task::Model>])
        .into_connection();

    let res = delete_task(&db, &DeleteTaskRequest { task_id: 1 }).await;

    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        ErrorInternalServerError("couldn't find task by id").to_string()
    );
}
#[actix_web::test]
async fn test_delete_request() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db))
            .service(delete_task_request),
    )
    .await;

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::DELETE)
        .set_json(DeleteTaskRequest { task_id: 1 })
        .uri("/task")
        .to_request();

    test::call_service(&app, req).await;
}
#[actix_web::test]
async fn test_delete_many_request() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 2,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db))
            .service(delete_tasks_request),
    )
    .await;

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::DELETE)
        .set_json([
            DeleteTaskRequest { task_id: 2 },
            DeleteTaskRequest { task_id: 1 },
        ])
        .uri("/tasks")
        .to_request();

    test::call_service(&app, req).await;
}
