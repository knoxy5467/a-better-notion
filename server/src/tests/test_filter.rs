use std::env;

use self::testcontainer_common_utils::DB;
use actix_web::test;
use common::{
    backend::{CreateTaskRequest, FilterRequest, FilterResponse, UpdateTaskRequest},
    Comparator, Filter, TaskProp, TaskPropVariant,
};
use testcontainer_common_utils::setup_db;

use super::*;
use sea_orm::MockDatabase;

#[actix_web::test]
async fn test_empty_filter() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres);
    let db_conn = db
        .append_query_results([vec![
            database::task::Model {
                id: 1,
                title: "heyo".to_owned(),
                completed: true,
                last_edited: chrono::NaiveDateTime::default(),
            },
            database::task::Model {
                id: 2,
                title: "heyo".to_owned(),
                completed: true,
                last_edited: chrono::NaiveDateTime::default(),
            },
        ]])
        .into_connection();
    let db_data: Data<DatabaseConnection> = Data::new(db_conn);
    let app = test::init_service(App::new().app_data(db_data).service(get_filter_request)).await;
    let req = test::TestRequest::default()
        .set_json(FilterRequest {
            filter: Filter::None,
        })
        .uri("/filter")
        .to_request();
    let resp: FilterResponse = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp[0], 1);
    assert_eq!(resp[1], 2);
}
#[actix_web::test]
async fn db_test() {
    env::set_var("RUST_LOG", "info");
    initialize_logger();
    info!("starting db");
    setup_db();
    let db = DB.get().unwrap();
    let db_conn = connect_to_database_exponential_backoff(
        4,
        "postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask".to_string(),
    )
    .await
    .unwrap();
    // run all my tests
    info!("create task");
    let id0: i32 = create_task(
        &db_conn,
        &CreateTaskRequest {
            name: "task 1".to_string(),
            completed: true,
            req_id: 1,
        },
    )
    .await
    .unwrap();
    info!("create task");
    let id1: i32 = create_task(
        &db_conn,
        &CreateTaskRequest {
            name: "task 2".to_string(),
            completed: true,
            req_id: 1,
        },
    )
    .await
    .unwrap();

    info!("update task");
    update_task(
        &db_conn,
        &UpdateTaskRequest {
            task_id: id0,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dogs".to_string(),
                value: TaskPropVariant::Number(1.0),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await
    .unwrap();
    info!("update task2");
    update_task(
        &db_conn,
        &UpdateTaskRequest {
            task_id: id1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dogs".to_string(),
                value: TaskPropVariant::Number(2.0),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await
    .unwrap();

    info!("filter");
    let res = filter(
        &db_conn,
        &FilterRequest {
            filter: Filter::Leaf {
                field: "dogs".to_string(),
                comparator: Comparator::EQ,
                immediate: TaskPropVariant::Number(1.0),
            },
        },
    )
    .await
    .unwrap();

    println!("{:?}", res);
    info!("shutting down db");
    // if tests are async you must await all of them before running below this will shut down the docker container
    db.stop();
}
