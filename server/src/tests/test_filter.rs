use std::env;

use self::testcontainer_common_utils::DB;

use super::*;
use common::{backend::*, Filter};
use testcontainer_common_utils::setup_db;
#[actix_web::test]
async fn filter_request() {
    use actix_web::test;
    use sea_orm::MockDatabase;

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
    // run all my tests

    info!("shutting down db");
    // if tests are async you must await all of them before running below this will shut down the docker container
    let db = DB.get().unwrap();
    db.stop();
}
