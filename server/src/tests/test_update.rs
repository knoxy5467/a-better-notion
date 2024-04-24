use super::*;
use actix_web::test;
use common::TaskProp;
use sea_orm::{MockDatabase, MockExecResult};

#[actix_web::test]
async fn task_id_fails() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_errors([sea_orm::error::DbErr::Query(
            sea_orm::error::RuntimeErr::Internal("test".to_string()),
        )])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 2,
            name: None,
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_err());
}
#[actix_web::test]
async fn name_changed() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "notdog".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: Some("dog".to_string()),
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
    //TODO: do a real test
}
#[actix_web::test]
async fn checked_changed() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "notdog".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: Some(false),
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}

#[actix_web::test]
async fn update_prop_string() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task_property::Model {
            task_id: 1,
            name: "name".to_string(),
            typ: "string".to_string(),
        }]])
        .append_query_results([[task_string_property::Model {
            task_id: 1,
            name: "name".to_string(),
            value: "value".to_string(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
                value: TaskPropVariant::String("newvalue".to_string()),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn create_prop_string() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([vec![] as Vec<task_property::Model>])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
                value: TaskPropVariant::String("value".to_string()),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn update_prop_num() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task_property::Model {
            task_id: 1,
            name: "name".to_string(),
            typ: "number".to_string(),
        }]])
        .append_query_results([[task_num_property::Model {
            task_id: 1,
            name: "name".to_string(),
            value: Decimal::from_f64(1.0).unwrap(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
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
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn create_prop_num() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([vec![] as Vec<task_property::Model>])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
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
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn update_prop_date() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task_property::Model {
            task_id: 1,
            name: "name".to_string(),
            typ: "date".to_string(),
        }]])
        .append_query_results([[task_date_property::Model {
            task_id: 1,
            name: "name".to_string(),
            value: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
                value: TaskPropVariant::Date(chrono::NaiveDateTime::default()),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn create_prop_date() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([vec![] as Vec<task_property::Model>])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
                value: TaskPropVariant::Date(chrono::NaiveDateTime::default()),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn update_prop_bool() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task_property::Model {
            task_id: 1,
            name: "name".to_string(),
            typ: "boolean".to_string(),
        }]])
        .append_query_results([[task_bool_property::Model {
            task_id: 1,
            name: "name".to_string(),
            value: true,
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
                value: TaskPropVariant::Boolean(false),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn create_prop_bool() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([vec![] as Vec<task_property::Model>])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
                value: TaskPropVariant::Boolean(true),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}

#[actix_web::test]
async fn prop_wrong_type() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task_property::Model {
            task_id: 1,
            name: "name".to_string(),
            typ: "number".to_string(),
        }]])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![TaskProp {
                name: "dog".to_string(),
                value: TaskPropVariant::String("newvalue".to_string()),
            }],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        ErrorInternalServerError("property dog has wrong type (expecting number)").to_string()
    )
}
#[actix_web::test]
async fn delete_prop() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task_property::Model {
            task_id: 1,
            name: "name".to_string(),
            typ: "number".to_string(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 2,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn delete_prop_bad_req() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([vec![] as Vec<task_property::Model>])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        ErrorInternalServerError("no property by name").to_string()
    )
}
#[actix_web::test]
async fn add_dep() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task::Model {
            id: 2,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![2],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn add_dep_doesnt_exist() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([vec![] as Vec<task::Model>])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![2],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        ErrorInternalServerError("task 1 can't depend on nonexistant task with id 2").to_string()
    );
}
#[actix_web::test]
async fn remove_dep() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[dependency::Model {
            task_id: 1,
            depends_on_id: 2,
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![2],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_ok());
}
#[actix_web::test]
async fn remove_dep_bad_req() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "title".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([vec![] as Vec<dependency::Model>])
        .into_connection();

    let res = update_task(
        &db,
        &UpdateTaskRequest {
            task_id: 1,
            name: None,
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![2],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        },
    )
    .await;

    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        ErrorInternalServerError("dependency couldn't be found").to_string()
    );
}
#[actix_web::test]
async fn test_task_update_request() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "notdog".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db))
            .service(update_task_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .set_json(UpdateTaskRequest {
            task_id: 1,
            name: Some("dog".to_string()),
            checked: None,
            props_to_add: vec![],
            props_to_remove: vec![],
            deps_to_add: vec![],
            deps_to_remove: vec![],
            scripts_to_add: vec![],
            scripts_to_remove: vec![],
            req_id: 0,
        })
        .uri("/task")
        .to_request();
    let resp: UpdateTaskResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.task_id, 1);
}
#[actix_web::test]
async fn test_tasks_update_request() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task::Model {
            id: 1,
            title: "notdog".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_query_results([[task::Model {
            id: 2,
            title: "notdog".to_string(),
            completed: true,
            last_edited: chrono::NaiveDateTime::default(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 2,
            rows_affected: 1,
        }])
        .append_exec_results([MockExecResult {
            last_insert_id: 2,
            rows_affected: 1,
        }])
        .into_connection();

    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db))
            .service(update_tasks_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .set_json([
            UpdateTaskRequest {
                task_id: 1,
                name: Some("dog".to_string()),
                checked: None,
                props_to_add: vec![],
                props_to_remove: vec![],
                deps_to_add: vec![],
                deps_to_remove: vec![],
                scripts_to_add: vec![],
                scripts_to_remove: vec![],
                req_id: 0,
            },
            UpdateTaskRequest {
                task_id: 2,
                name: Some("dog".to_string()),
                checked: None,
                props_to_add: vec![],
                props_to_remove: vec![],
                deps_to_add: vec![],
                deps_to_remove: vec![],
                scripts_to_add: vec![],
                scripts_to_remove: vec![],
                req_id: 1,
            },
        ])
        .uri("/tasks")
        .to_request();
    let resp: UpdateTasksResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp[0].task_id, 1);
    assert_eq!(resp[1].task_id, 2);
}
