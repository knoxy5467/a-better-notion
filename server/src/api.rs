use actix_web::{get, put, web, HttpResponse, Responder, ResponseError, Result};
use common::backend::*;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, DbErr, Set, Unset};
use std::fmt;

use crate::database::task;
// Define a new type that wraps DbErr
pub struct MyDbErr(DbErr);

impl fmt::Debug for MyDbErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for MyDbErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

// Implement ResponseError for the new type
impl ResponseError for MyDbErr {
    fn error_response(&self) -> HttpResponse {
        // Customize the HTTP response based on the error
        HttpResponse::InternalServerError().json("Internal server error")
    }
}

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
        .map_err(|e| actix_web::error::ErrorInternalServerError(MyDbErr(e)))?;
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
        .map_err(|e| actix_web::error::ErrorInternalServerError(MyDbErr(e)))?;
    Ok(web::Json(result_task.last_insert_id))
}
