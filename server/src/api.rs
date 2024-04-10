use crate::database::{task, task_property, task_string_property};
#[allow(unused)]
use actix_web::{delete, get, post, put, web, Responder, Result};
use common::{backend::*, TaskProp, TaskPropVariant};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Condition, Set};
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

/// post /task endpoint creates a single task
async fn create_task(
    data: &web::Data<DatabaseConnection>,
    req: &CreateTaskRequest,
) -> Result<web::Json<CreateTaskResponse>> {
    let task_model = task::ActiveModel {
        id: NotSet,
        title: Set(req.name.clone()),
        completed: Set(req.completed),
        last_edited: Set(chrono::Local::now().naive_local()),
    };
    let result_task = task::Entity::insert(task_model)
        .exec(data.as_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("task not inserted {}", e))
        })?; //TODO handle this error better, for example for unique constraint violation
    Ok(web::Json(result_task.last_insert_id as CreateTaskResponse))
}

#[post("/task")]
async fn create_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<CreateTaskRequest>,
) -> Result<web::Json<CreateTaskResponse>> {
    create_task(&data, &req).await
}
/// post /tasks endpoint cretes multiple tasks
#[post("/tasks")]
async fn create_tasks_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<CreateTasksRequest>,
) -> Result<web::Json<CreateTasksResponse>> {
    let mut res: CreateTasksResponse = Vec::new();
    for taskreq in req.iter() {
        let id = create_task(&data, taskreq)
            .await
            .unwrap_or(web::Json(-1))
            .to_owned();
        res.push(id)
    }
    Ok(web::Json(res))
}

/// put /task updates one task
async fn update_task(
    data: &web::Data<DatabaseConnection>,
    req: &UpdateTaskRequest,
) -> Result<web::Json<UpdateTaskResponse>> {
    let task = task::Entity::find_by_id(req.task_id)
        .one(data.as_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("couldn't fetch tasks: {}", e))
        })?;
    let mut task: task::ActiveModel = task.unwrap().into();
    if req.name.is_some() {
        task.title = Set(req.name.to_owned().unwrap());
    }
    if req.checked.is_some() {
        task.completed = Set(req.checked.unwrap());
    }
    for prop in req.props_to_add.iter() {
        //create the new property
        match &prop.value {
            TaskPropVariant::String(val) => {
                let newprop = task_string_property::ActiveModel {
                    task_id: Set(req.task_id),
                    name: Set(prop.name.to_owned()),
                    value: Set(val.to_string()),
                };
            }
            TaskPropVariant::Number(val) => {}
            TaskPropVariant::Date(val) => {}
            TaskPropVariant::Boolean(val) => {}
        };
    }
    for _prop in req.props_to_remove.iter() {
        todo!("remove task_property and typed tasks");
    }
    for _dep in req.deps_to_add.iter() {
        //TODO: implement deps
    }
    for _dep in req.deps_to_remove.iter() {
        //TODO: implement deps
    }
    for _script in req.scripts_to_add.iter() {
        //TODO: implement scripts
    }
    for _script in req.scripts_to_remove.iter() {
        //TODO: implement scripts
    }

    Ok(web::Json(1))
}
#[put("/task")]
async fn update_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<UpdateTaskRequest>,
) -> Result<web::Json<UpdateTaskResponse>> {
    update_task(&data, &req).await
}
#[put("/tasks")]
async fn update_tasks_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<UpdateTasksRequest>,
) -> Result<web::Json<UpdateTasksResponse>> {
    let mut res: UpdateTasksResponse = Vec::new();
    for taskreq in req.iter() {
        let id = update_task(&data, taskreq)
            .await
            .unwrap_or(web::Json(-1))
            .to_owned();
        res.push(id);
    }
    Ok(web::Json(res))
}

async fn delete_task(
    data: &web::Data<DatabaseConnection>,
    req: &DeleteTaskRequest,
) -> Result<web::Json<DeleteTaskResponse>> {
    task::Entity::find_by_id(req.task_id)
        .one(data.as_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("couldn't find task: {}", e))
        })?
        .unwrap()
        .delete(data.as_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("couldn't delete task: {}", e))
        })?;

    Ok(web::Json(()))
}

#[delete("/task")]
async fn delete_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<DeleteTaskRequest>,
) -> Result<web::Json<DeleteTaskResponse>> {
    delete_task(&data, &req).await
}
#[delete("/tasks")]
async fn delete_tasks_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<DeleteTasksRequest>,
) -> Result<web::Json<DeleteTasksResponse>> {
    for task in req.iter() {
        delete_task(&data, task);
    }
    Ok(web::Json(()))
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
