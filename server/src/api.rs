use crate::database::task;
#[allow(unused)]
use actix_web::{get, put, web, Responder, Result};
use common::backend::*;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};

/// get /task endpoint for retrieving a single TaskShort
#[get("/task")]
async fn get_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<ReadTaskShortRequest>,
) -> Result<impl Responder> {
    println!("requesting task id {}", req.task_id);
    let db = data;
    let task = task::Entity::find_by_id(req.task_id)
        .one(db.as_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("SQL error: {}", e)))?; // TODO handle this error better, if it does not exist then it should be a http 204 error
    match task {
        Some(model) => Ok(web::Json(ReadTaskShortResponse {
            task_id: model.id,
            name: model.title,
            completed: model.completed,
            props: Vec::new(),   //TODO 26mar24 Mrknox: implement properties
            deps: Vec::new(),    //TODO 26mar24 Mrknox: implement dependencies
            scripts: Vec::new(), //TODO 26mar24 Mrknox: implement scripts
            last_edited: model.last_edited,
        })),
        None => Err(actix_web::error::ErrorNotFound("task not found by ID")),
    }
}

/// get /tasks endpoint for retrieving some number of TaskShorts
#[get("/tasks")]
async fn get_tasks_request(req: web::Json<Vec<ReadTaskShortRequest>>) -> Result<impl Responder> {
    // do diesel stuff here
    Ok(web::Json(vec![ReadTaskShortResponse {
        task_id: req[0].task_id,
        name: "heyo".to_string(),
        completed: false,
        props: Vec::new(),
        deps: Vec::new(),
        scripts: Vec::new(),
        last_edited: chrono::NaiveDateTime::default(),
    }]))
}

#[put("/task")]
async fn create_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<CreateTaskRequest>,
) -> Result<impl Responder> {
    let db = data;
    let task_model = task::ActiveModel {
        id: NotSet,
        title: Set(req.name.clone()),
        completed: Set(req.completed),
        last_edited: Set(chrono::Local::now().naive_local()),
    };
    let result_task = task::Entity::insert(task_model)
        .exec(db.as_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("task not inserted {}", e))
        })?; //TODO handle this error better, for example for unique constraint violation
    Ok(web::Json(result_task.last_insert_id as CreateTaskResponse))
}

#[cfg(test)]
mod tests {
    use std::vec;

    use actix_web::{dev::ServiceResponse, http::StatusCode};

    use super::*;

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
    async fn put_task_fails_with_bad_request() {
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
}
