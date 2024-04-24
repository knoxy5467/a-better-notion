use super::*;
use actix_web::test;
use sea_orm::MockDatabase;

#[actix_web::test]
async fn get_bad_id() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([vec![] as Vec<task::Model>])
        .into_connection();

    let res = get_property_or_err(&db, &"title".to_string(), 1).await;
    assert!(res.is_err());
}

#[actix_web::test]
async fn get_string() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
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
        .into_connection();

    let res = get_property_or_err(&db, &"name".to_string(), 1).await;
    assert!(res.is_ok());
    assert_eq!(
        res.unwrap(),
        Some(TaskPropVariant::String("value".to_string()))
    );
}
#[actix_web::test]
async fn get_number() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
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
        .into_connection();

    let res = get_property_or_err(&db, &"name".to_string(), 1).await;
    assert!(res.is_ok());
    assert_eq!(
        res.unwrap(),
        Some(TaskPropVariant::Number(
            rust_decimal::Decimal::from_f64(1.0).unwrap()
        ))
    );
}
#[actix_web::test]
async fn get_date() {
    let date = chrono::NaiveDateTime::default();
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task_property::Model {
            task_id: 1,
            name: "name".to_string(),
            typ: "date".to_string(),
        }]])
        .append_query_results([[task_date_property::Model {
            task_id: 1,
            name: "name".to_string(),
            value: date,
        }]])
        .into_connection();

    let res = get_property_or_err(&db, &"name".to_string(), 1).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), Some(TaskPropVariant::Date(date)));
}
#[actix_web::test]
async fn get_bool() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([[task_property::Model {
            task_id: 1,
            name: "name".to_string(),
            typ: "boolean".to_string(),
        }]])
        .append_query_results([[task_bool_property::Model {
            task_id: 1,
            name: "boolean".to_string(),
            value: true,
        }]])
        .into_connection();

    let res = get_property_or_err(&db, &"name".to_string(), 1).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), Some(TaskPropVariant::Boolean(true)));
}

#[actix_web::test]
async fn get_prop_dne() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results([vec![] as Vec<task_property::Model>])
        .into_connection();

    let res = get_property_or_err(&db, &"name".to_string(), 1).await;
    assert!(res.is_err());
}
#[actix_web::test]
async fn test_property_request() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
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
        .into_connection();

    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db))
            .service(get_property_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .set_json(PropertyRequest {
            task_id: 1,
            properties: vec!["name".to_string()],
            req_id: 0,
        })
        .uri("/prop")
        .to_request();

    let resp: PropertyResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.res[0].name, "name".to_owned());
    assert_eq!(
        resp.res[0].value,
        Some(TaskPropVariant::String("value".to_string()))
    )
}
#[actix_web::test]
async fn test_properties_request() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
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
        .into_connection();

    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(db))
            .service(get_properties_request),
    )
    .await;
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .set_json(PropertiesRequest {
            task_ids: vec![1],
            properties: vec!["name".to_string()],
            req_id: 0,
        })
        .uri("/props")
        .to_request();

    let resp: PropertiesResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.res[0].name, "name".to_owned());
    assert_eq!(
        resp.res[0].values,
        vec![Some(TaskPropVariant::String("value".to_string()))]
    )
}
