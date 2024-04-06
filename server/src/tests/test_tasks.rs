use super::*;
use actix_web::{dev::ServiceResponse, http::StatusCode, web};
use common::backend::*;
use sea_orm::MockExecResult;
use std::vec;

#[actix_web::test]
async fn get_task_fails_with_bad_request() {
    use actix_web::test;
    use sea_orm::MockDatabase;

    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres);
    let db_conn = db
        .append_query_errors([sea_orm::error::DbErr::Query(
            sea_orm::error::RuntimeErr::Internal("test".to_string()),
        )])
        .into_connection();
    let db_data: web::Data<DatabaseConnection> = web::Data::new(db_conn);
    let app = test::init_service(
        actix_web::App::new()
            .app_data(db_data)
            .service(get_task_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .set_json(ReadTaskShortRequest { task_id: 2 })
        .uri("/task")
        .to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
#[actix_web::test]
async fn get_task_succeeds_with_good_request() {
    use actix_web::test;
    use sea_orm::MockDatabase;

    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres);
    let db_conn = db
        .append_query_results([vec![database::task::Model {
            id: 1,
            title: "test".to_string(),
            completed: false,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .into_connection();
    let db_data: Data<DatabaseConnection> = Data::new(db_conn);
    let app = test::init_service(App::new().app_data(db_data).service(get_task_request)).await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .set_json(ReadTaskShortRequest { task_id: 1 })
        .uri("/task")
        .to_request();
    let resp: ReadTaskShortResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.task_id, 1);
}
#[actix_web::test]
async fn get_tasks_request_succeeds_with_good_request() {
    use actix_web::test;
    use sea_orm::MockDatabase;

    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres);
    let db_conn = db
        .append_query_results([
            vec![database::task::Model {
                id: 1,
                title: "test".to_string(),
                completed: false,
                last_edited: chrono::NaiveDateTime::default(),
            }],
            vec![database::task::Model {
                id: 2,
                title: "test2".to_string(),
                completed: false,
                last_edited: chrono::NaiveDateTime::default(),
            }],
            vec![],
        ])
        .into_connection();
    let db_data: Data<DatabaseConnection> = Data::new(db_conn);
    let app = test::init_service(App::new().app_data(db_data).service(get_tasks_request)).await;
    let req = test::TestRequest::default()
        .set_json(vec![
            ReadTaskShortRequest { task_id: 1 },
            ReadTaskShortRequest { task_id: 2 },
            ReadTaskShortRequest { task_id: 3 },
        ])
        .uri("/tasks")
        .to_request();
    let resp: ReadTasksShortResponse = test::call_and_read_body_json(&app, req).await;

    assert!(resp[0].as_ref().is_ok_and(|a| a.task_id == 1));
    assert!(resp[1].as_ref().is_ok_and(|a| a.task_id == 2));
    assert!(resp[2].is_err());
}
#[actix_web::test]
async fn insert_task_fails_with_bad_request() {
    use actix_web::test;
    use sea_orm::MockDatabase;
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
        .method(actix_web::http::Method::PUT)
        .set_json(CreateTaskRequest {
            name: "test".to_string(),
            completed: false,
            properties: vec![],
            dependencies: vec![],
        })
        .uri("/task")
        .to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_web::test]
async fn insert_task_succeeds_with_good_request() {
    use actix_web::test;
    use sea_orm::MockDatabase;
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres);
    let db_conn = db
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results([vec![database::task::Model {
            id: 1,
            title: "test".to_string(),
            completed: false,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .into_connection();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(db_conn))
            .service(create_task_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .set_json(CreateTaskRequest {
            name: "test".to_string(),
            completed: false,
            properties: vec![],
            dependencies: vec![],
        })
        .uri("/task")
        .to_request();
    let response: CreateTaskResponse = test::call_and_read_body_json(&app, req).await;
    env_logger::builder().is_test(true).try_init().unwrap();
    assert_eq!(response, 1);
}
