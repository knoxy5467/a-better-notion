use std::env;

#[allow(unused_imports)]
use crate::connect_to_database_exponential_backoff;
#[allow(unused_imports)]
use crate::{database, initialize_logger, testcontainer_common_utils};
#[allow(unused_imports)]
use actix_web::{test, web::Data, App};
use chrono::NaiveDate;
use common::backend::*;
use common::*;
use log::info;
use testcontainer_common_utils::setup_db;
use testcontainer_common_utils::DB;

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

macro_rules! simple_test {
    ($name:expr, $comp:expr, $imm:expr, $res:expr, $db_conn:expr) => {
        let res = filter(
            $db_conn,
            &FilterRequest {
                filter: Filter::Leaf {
                    field: $name.to_string(),
                    comparator: $comp,
                    immediate: $imm,
                },
            },
        )
        .await
        .unwrap();

        println!("comparing {} {}", res[0], $res);
        assert_eq!(res[0], $res);
        assert!(res.len() == 1);
    };
}
macro_rules! simple_primitive_test {
    ($name:expr, $comp:expr, $imm:expr, $res:expr, $db_conn:expr) => {
        let res = filter(
            $db_conn,
            &FilterRequest {
                filter: Filter::LeafPrimitive {
                    field: $name,
                    comparator: $comp,
                    immediate: $imm,
                },
            },
        )
        .await
        .unwrap();

        println!("comparing {} {}", res[0], $res);
        assert_eq!(res[0], $res);
        assert!(res.len() == 1);
    };
}

macro_rules! simple_make {
    ($val1:expr, $val2:expr, $name:expr, $db_conn:expr, $id0:ident, $id1:ident) => {
        let $id0 = create_task(
            $db_conn,
            &CreateTaskRequest {
                name: "task 1".to_string(),
                completed: true,
                req_id: 1,
            },
        )
        .await
        .unwrap();
        let $id1 = create_task(
            $db_conn,
            &CreateTaskRequest {
                name: "task 2".to_string(),
                completed: true,
                req_id: 1,
            },
        )
        .await
        .unwrap();
        update_task(
            $db_conn,
            &UpdateTaskRequest {
                task_id: $id0,
                name: None,
                checked: None,
                props_to_add: vec![TaskProp {
                    name: $name.to_string(),
                    value: $val1,
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
        update_task(
            $db_conn,
            &UpdateTaskRequest {
                task_id: $id1,
                name: None,
                checked: None,
                props_to_add: vec![TaskProp {
                    name: $name.to_string(),
                    value: $val2,
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
    };
}
macro_rules! super_make {
    ($id:ident, $db_conn:expr, $name:expr, $completed:expr, $($pair:expr),*) => {
        let $id = create_task($db_conn, &CreateTaskRequest {
            name: $name.to_string(),
            completed: $completed,
            req_id: 0,
        }).await.unwrap();
        update_task(
            $db_conn,
            &UpdateTaskRequest {
                task_id: $id,
                name: None,
                checked: None,
                props_to_add: vec![$(
                    TaskProp {
                        name: $pair.0.to_string(),
                        value: $pair.1,
                    },
                )*],
                props_to_remove: vec![],
                deps_to_add: vec![],
                deps_to_remove: vec![],
                scripts_to_add: vec![],
                scripts_to_remove: vec![],
                req_id: 0,
            }
        ).await.unwrap();
    }
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
    super_make!(
        id0,
        &db_conn,
        "a dude",
        true,
        ("dogs", TaskPropVariant::Number(1.0))
    );
    super_make!(
        id1,
        &db_conn,
        "b not that",
        false,
        ("dogs", TaskPropVariant::Number(2.0))
    );

    simple_test!(
        "dogs",
        Comparator::LT,
        TaskPropVariant::Number(1.5),
        id0,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::LEQ,
        TaskPropVariant::Number(1.0),
        id0,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::GT,
        TaskPropVariant::Number(1.5),
        id1,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::GEQ,
        TaskPropVariant::Number(2.0),
        id1,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::EQ,
        TaskPropVariant::Number(1.0),
        id0,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::NEQ,
        TaskPropVariant::Number(1.0),
        id1,
        &db_conn
    );

    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::LT,
        TaskPropVariant::String("b".to_string()),
        id0,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::LEQ,
        TaskPropVariant::String("b".to_string()),
        id0,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::GT,
        TaskPropVariant::String("b".to_string()),
        id1,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::GEQ,
        TaskPropVariant::String("b".to_string()),
        id1,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::EQ,
        TaskPropVariant::String("a dude".to_string()),
        id0,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::NEQ,
        TaskPropVariant::String("a dude".to_string()),
        id1,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::CONTAINS,
        TaskPropVariant::String("dude".to_string()),
        id0,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::NOTCONTAINS,
        TaskPropVariant::String("dude".to_string()),
        id1,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::TITLE,
        Comparator::LIKE,
        TaskPropVariant::String("%dude%".to_string()),
        id0,
        &db_conn
    );

    simple_primitive_test!(
        PrimitiveField::COMPLETED,
        Comparator::EQ,
        TaskPropVariant::Boolean(true),
        id0,
        &db_conn
    );
    simple_primitive_test!(
        PrimitiveField::COMPLETED,
        Comparator::NEQ,
        TaskPropVariant::Boolean(false),
        id0,
        &db_conn
    );

    super_make!(
        id2,
        &db_conn,
        "task1",
        true,
        ("dogs", TaskPropVariant::Boolean(true))
    );
    super_make!(
        id3,
        &db_conn,
        "task2",
        true,
        ("dogs", TaskPropVariant::Boolean(false))
    );
    simple_test!(
        "dogs",
        Comparator::EQ,
        TaskPropVariant::Boolean(true),
        id2,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::NEQ,
        TaskPropVariant::Boolean(false),
        id2,
        &db_conn
    );
    simple_make!(
        TaskPropVariant::String("a dude".to_string()),
        TaskPropVariant::String("c not that".to_string()),
        "dogs",
        &db_conn,
        id4,
        id5
    );
    simple_test!(
        "dogs",
        Comparator::LT,
        TaskPropVariant::String("b".to_string()),
        id4,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::LEQ,
        TaskPropVariant::String("b".to_string()),
        id4,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::GT,
        TaskPropVariant::String("b".to_string()),
        id5,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::GEQ,
        TaskPropVariant::String("b".to_string()),
        id5,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::EQ,
        TaskPropVariant::String("a dude".to_string()),
        id4,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::NEQ,
        TaskPropVariant::String("a dude".to_string()),
        id5,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::CONTAINS,
        TaskPropVariant::String("dude".to_string()),
        id4,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::NOTCONTAINS,
        TaskPropVariant::String("dude".to_string()),
        id5,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::LIKE,
        TaskPropVariant::String("%dude%".to_string()),
        id4,
        &db_conn
    );
    simple_make!(
        TaskPropVariant::Date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        ),
        TaskPropVariant::Date(
            NaiveDate::from_ymd_opt(2024, 2, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        ),
        "dogs",
        &db_conn,
        id6,
        id7
    );
    simple_test!(
        "dogs",
        Comparator::LT,
        TaskPropVariant::Date(
            NaiveDate::from_ymd_opt(2024, 1, 5)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        ),
        id6,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::LEQ,
        TaskPropVariant::Date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        ),
        id6,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::GT,
        TaskPropVariant::Date(
            NaiveDate::from_ymd_opt(2024, 1, 5)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        ),
        id7,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::GEQ,
        TaskPropVariant::Date(
            NaiveDate::from_ymd_opt(2024, 2, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        ),
        id7,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::EQ,
        TaskPropVariant::Date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        ),
        id6,
        &db_conn
    );
    simple_test!(
        "dogs",
        Comparator::NEQ,
        TaskPropVariant::Date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        ),
        id7,
        &db_conn
    );

    super_make!(
        id8,
        &db_conn,
        "a dude",
        false,
        ("dog2s", TaskPropVariant::Number(3.0)),
        ("dog3s", TaskPropVariant::Boolean(true))
    );
    super_make!(
        id9,
        &db_conn,
        "b not that",
        true,
        ("dog2s", TaskPropVariant::Number(2.0)),
        ("dog3s", TaskPropVariant::Boolean(false))
    );
    let mut res = filter(
        &db_conn,
        &FilterRequest {
            filter: Filter::Operator {
                op: common::Operator::AND,
                childs: vec![
                    Filter::Leaf {
                        field: "dog2s".to_string(),
                        comparator: Comparator::GT,
                        immediate: TaskPropVariant::Number(1.0),
                    },
                    Filter::Leaf {
                        field: "dog3s".to_string(),
                        comparator: Comparator::EQ,
                        immediate: TaskPropVariant::Boolean(true),
                    },
                ],
            },
        },
    )
    .await
    .unwrap();

    assert_eq!(res[0], id8);
    assert!(res.len() == 1);

    res = filter(
        &db_conn,
        &FilterRequest {
            filter: Filter::Operator {
                op: common::Operator::OR,
                childs: vec![
                    Filter::Leaf {
                        field: "dog2s".to_string(),
                        comparator: Comparator::EQ,
                        immediate: TaskPropVariant::Number(3.0),
                    },
                    Filter::Leaf {
                        field: "dog3s".to_string(),
                        comparator: Comparator::EQ,
                        immediate: TaskPropVariant::Boolean(false),
                    },
                ],
            },
        },
    )
    .await
    .unwrap();

    assert!(res.contains(&id8));
    assert!(res.contains(&id9));
    assert!(res.len() == 2);

    res = filter(
        &db_conn,
        &FilterRequest {
            filter: Filter::Operator {
                op: common::Operator::NOT,
                childs: vec![Filter::Leaf {
                    field: "dog3s".to_string(),
                    comparator: Comparator::EQ,
                    immediate: TaskPropVariant::Boolean(false),
                }],
            },
        },
    )
    .await
    .unwrap();
    assert!(!res.contains(&id9));

    info!("shutting down db");
    // if tests are async you must await all of them before running below this will shut down the docker container
    db.stop();
}
