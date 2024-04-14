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
async fn get_tasks_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<ReadTasksShortRequest>,
) -> Result<impl Responder> {
    let mut res: ReadTasksShortResponse = Vec::new();

    for taskreq in req.iter() {
        let task = task::Entity::find_by_id(taskreq.task_id)
            .one(data.as_ref())
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("couldn't fetch tasks: {}", e))
            })?;
        match task {
            Some(model) => res.push(Ok(ReadTaskShortResponse {
                task_id: model.id,
                name: model.title,
                completed: model.completed,
                props: Vec::new(),
                deps: Vec::new(),
                scripts: Vec::new(),
                last_edited: model.last_edited,
            })),
            None => res.push(Err("task not found by ID".to_string())),
        }
    }

    Ok(web::Json(res))
}

/// get /filter endpoint for retrieving some number of TaskShorts
#[allow(unused_variables)]
#[get("/filter")]
async fn get_filter_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<FilterRequest>,
) -> Result<impl Responder> {
    //TODO: construct filter

    let tasks: Vec<task::Model> = task::Entity::find().all(data.as_ref()).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("couldn't filter tasks: {}", e))
    })?;

    Ok(web::Json(
        tasks.iter().map(|a| a.id).collect::<FilterResponse>(),
    ))
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
