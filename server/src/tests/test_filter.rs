use std::env;

use self::testcontainer_common_utils::DB;

use super::*;
use crate::database::*;
use actix_web::{error::ErrorInternalServerError, test};
use common::{backend::*, Comparator, Filter, TaskProp, TaskPropVariant};
use sea_orm::MockDatabase;
use testcontainer_common_utils::setup_db;

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
            req_id: 0,
        })
        .uri("/filter")
        .to_request();
    let resp: FilterResponse = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.tasks[0], 1);
    assert_eq!(resp.tasks[1], 2);
}
#[actix_web::test]
async fn db_test() {
    env::set_var("RUST_LOG", "info");
    initialize_logger();
    info!("starting db");
    setup_db();
    // run all my tests
    let db = DB.get().unwrap();

    let ids: [TaskID] = create_tasks_request(
        db,
        vec![
            CreateTaskRequest {
                name: "task 1".to_string(),
                completed: true,
                req_id: 1,
            },
            CreateTaskRequest {
                name: "task 2".to_string(),
                completed: true,
                req_id: 1,
            },
        ],
    )
    .await
    .unwrap()
    .to_owned()
    .map(|x| x.task_id);

    update_tasks_request(
        db,
        vec![
            UpdateTaskRequest {
                task_id: ids[0],
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
            UpdateTaskRequest {
                task_id: ids[1],
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
        ],
    )
    .await
    .unwrap();

    let res = filter_request(
        db,
        FilterRequest {
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
