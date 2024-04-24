use crate::database::*;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
#[allow(unused)]
use actix_web::{delete, get, post, put, web, Responder, Result};
use common::{backend::*, TaskID, TaskPropVariant};
use log::info;
use rust_decimal::prelude::FromPrimitive;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Condition, IntoActiveModel, Set};
// get /task endpoint for retrieving a single TaskShort
#[get("/task")]
async fn get_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<ReadTaskShortRequest>,
) -> Result<impl Responder> {
    let db = data;
    let task = task::Entity::find_by_id(req.task_id)
        .one(db.as_ref())
        .await
        .map_err(|e| ErrorInternalServerError(format!("SQL error: {}", e)))?; // TODO handle this error better, if it does not exist then it should be a http 204 error
    match task {
        Some(model) => Ok(web::Json(ReadTaskShortResponse {
            task_id: model.id,
            name: model.title,
            completed: model.completed,
            props: Vec::new(),   //TODO 26mar24 Mrknox: implement properties
            deps: Vec::new(),    //TODO 26mar24 Mrknox: implement dependencies
            scripts: Vec::new(), //TODO 26mar24 Mrknox: implement scripts
            last_edited: model.last_edited,
            req_id: req.req_id,
        })),
        None => Err(ErrorNotFound("task not found by ID")),
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
            .map_err(|e| ErrorInternalServerError(format!("couldn't fetch tasks: {}", e)))?;
        match task {
            Some(model) => res.push(Ok(ReadTaskShortResponse {
                task_id: model.id,
                name: model.title,
                completed: model.completed,
                props: Vec::new(),
                deps: Vec::new(),
                scripts: Vec::new(),
                last_edited: model.last_edited,
                req_id: taskreq.req_id,
            })),
            None => res.push(Err("task not found by ID".to_string())),
        }
    }

    Ok(web::Json(res))
}

/// post /task endpoint creates a single task
async fn create_task(db: &DatabaseConnection, req: &CreateTaskRequest) -> Result<TaskID> {
    let task_model = task::ActiveModel {
        id: NotSet,
        title: Set(req.name.clone()),
        completed: Set(req.completed),
        last_edited: Set(chrono::Local::now().naive_local()),
    };
    let result_task = task_model
        .insert(db)
        .await
        .map_err(|e| ErrorInternalServerError(format!("task not inserted: {}", e)))?; //TODO handle this error better, for example for unique constraint violation
    Ok(result_task.id)
}

#[post("/task")]
async fn create_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<CreateTaskRequest>,
) -> Result<web::Json<CreateTaskResponse>> {
    info!("create_task_request: {:?}", req);
    let id = create_task(&data, &req).await?;
    Ok(web::Json(CreateTaskResponse {
        task_id: id,
        req_id: req.req_id,
    }))
}
/// post /tasks endpoint cretes multiple tasks
#[post("/tasks")]
async fn create_tasks_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<CreateTasksRequest>,
) -> Result<web::Json<CreateTasksResponse>> {
    let mut res: CreateTasksResponse = Vec::new();
    for taskreq in req.iter() {
        let id = create_task(&data, taskreq).await?;
        res.push(CreateTaskResponse {
            task_id: id,
            req_id: taskreq.req_id,
        });
    }
    Ok(web::Json(res))
}

/// put /task updates one task
async fn update_task(db: &DatabaseConnection, req: &UpdateTaskRequest) -> Result<TaskID> {
    let task = task::Entity::find_by_id(req.task_id)
        .one(db)
        .await
        .map_err(|e| ErrorInternalServerError(format!("couldn't fetch tasks: {}", e)))?
        .ok_or("no task by id")
        .map_err(ErrorInternalServerError)?;

    let mut task: task::ActiveModel = task.into();
    if req.name.is_some() {
        task.title = Set(req.name.to_owned().unwrap());
    }
    if req.checked.is_some() {
        task.completed = Set(req.checked.unwrap());
    }
    for prop in req.props_to_add.iter() {
        let model = task_property::Entity::find()
            .filter(
                Condition::all()
                    .add(task_property::Column::TaskId.eq(req.task_id))
                    .add(task_property::Column::Name.eq(req.name.to_owned())),
            )
            .one(db)
            .await
            .map_err(|e| ErrorInternalServerError(format!("couldn't fetch property: {}", e)))?;

        //model exists but type is wrong
        if model
            .as_ref()
            .is_some_and(|m| m.typ != prop.value.type_string())
        {
            return Err(ErrorInternalServerError(format!(
                "property {} has wrong type (expecting {})",
                prop.name,
                model.unwrap().typ
            )));
        }

        //model already exists and we can just update
        if model.is_some() {
            match &prop.value {
                TaskPropVariant::String(val) => {
                    let p = task_string_property::Entity::find()
                        .filter(
                            Condition::all()
                                .add(task_string_property::Column::TaskId.eq(req.task_id))
                                .add(task_string_property::Column::Name.eq(prop.name.to_owned())),
                        )
                        .one(db)
                        .await
                        .map_err(|e| {
                            ErrorInternalServerError(format!("couldn't fetch property: {}", e))
                        })?
                        .ok_or("couldn't find property by name")
                        .map_err(ErrorInternalServerError)?;
                    let mut p = p.into_active_model();
                    p.value = Set(val.to_owned());
                }
                TaskPropVariant::Date(val) => {
                    let p = task_date_property::Entity::find()
                        .filter(
                            Condition::all()
                                .add(task_date_property::Column::TaskId.eq(req.task_id))
                                .add(task_date_property::Column::Name.eq(prop.name.to_owned())),
                        )
                        .one(db)
                        .await
                        .map_err(|e| {
                            ErrorInternalServerError(format!("couldn't fetch property: {}", e))
                        })?
                        .ok_or("couldn't find property by name")
                        .map_err(ErrorInternalServerError)?;
                    let mut p = p.into_active_model();
                    p.value = Set(*val);
                }
                TaskPropVariant::Number(val) => {
                    let p = task_num_property::Entity::find()
                        .filter(
                            Condition::all()
                                .add(task_num_property::Column::TaskId.eq(req.task_id))
                                .add(task_num_property::Column::Name.eq(prop.name.to_owned())),
                        )
                        .one(db)
                        .await
                        .map_err(|e| {
                            ErrorInternalServerError(format!("couldn't fetch property: {}", e))
                        })?
                        .ok_or("couldn't find property by name")
                        .map_err(ErrorInternalServerError)?;
                    let mut p = p.into_active_model();
                    p.value = Set(val.clone());
                }
                TaskPropVariant::Boolean(val) => {
                    let p = task_bool_property::Entity::find()
                        .filter(
                            Condition::all()
                                .add(task_bool_property::Column::TaskId.eq(req.task_id))
                                .add(task_bool_property::Column::Name.eq(prop.name.to_owned())),
                        )
                        .one(db)
                        .await
                        .map_err(|e| {
                            ErrorInternalServerError(format!("couldn't fetch property: {}", e))
                        })?
                        .ok_or("couldn't find property by name")
                        .map_err(ErrorInternalServerError)?;
                    let mut p = p.into_active_model();
                    p.value = Set(*val);
                }
            }

            continue;
        }

        //otherwise we create a new property
        match &prop.value {
            TaskPropVariant::String(val) => {
                task_string_property::Entity::insert(task_string_property::ActiveModel {
                    task_id: Set(req.task_id),
                    name: Set(prop.name.to_owned()),
                    value: Set(val.to_string()),
                })
                .exec(db)
                .await
                .map_err(|e| {
                    ErrorInternalServerError(format!("couldn't create property: {}", e))
                })?;
            }
            TaskPropVariant::Number(val) => {
                task_num_property::Entity::insert(task_num_property::ActiveModel {
                    task_id: Set(req.task_id),
                    name: Set(prop.name.to_owned()),
                    value: Set(val.clone()),
                })
                .exec(db)
                .await
                .map_err(|e| {
                    ErrorInternalServerError(format!("couldn't create property: {}", e))
                })?;
            }
            TaskPropVariant::Date(val) => {
                task_date_property::Entity::insert(task_date_property::ActiveModel {
                    task_id: Set(req.task_id),
                    name: Set(prop.name.to_owned()),
                    value: Set(val.to_owned()),
                })
                .exec(db)
                .await
                .map_err(|e| {
                    ErrorInternalServerError(format!("couldn't create property: {}", e))
                })?;
            }
            TaskPropVariant::Boolean(val) => {
                task_bool_property::Entity::insert(task_bool_property::ActiveModel {
                    task_id: Set(req.task_id),
                    name: Set(prop.name.to_owned()),
                    value: Set(*val),
                })
                .exec(db)
                .await
                .map_err(|e| {
                    ErrorInternalServerError(format!("couldn't create property: {}", e))
                })?;
            }
        };
    }
    for prop in req.props_to_remove.iter() {
        task_property::Entity::find()
            .filter(
                Condition::all()
                    .add(task_property::Column::TaskId.eq(req.task_id))
                    .add(task_property::Column::Name.eq(prop.name.clone())),
            )
            .one(db)
            .await
            .map_err(|e| ErrorInternalServerError(format!("couldn't fetch property: {}", e)))?
            .ok_or("no property by name")
            .map_err(ErrorInternalServerError)?
            .delete(db)
            .await
            .map_err(ErrorInternalServerError)?;
    }
    for dep in req.deps_to_add.iter() {
        if task::Entity::find_by_id(*dep)
            .one(db)
            .await
            .map_err(|e| ErrorInternalServerError(format!("couldn't fetch task: {}", e)))?
            .is_none()
        {
            return Err(ErrorInternalServerError(format!(
                "task {} can't depend on nonexistant task with id {}",
                req.task_id, dep
            )));
        }

        dependency::Entity::insert(dependency::ActiveModel {
            task_id: Set(req.task_id),
            depends_on_id: Set(*dep),
        })
        .exec(db)
        .await
        .map_err(|e| ErrorInternalServerError(format!("couldn't create dependancy: {}", e)))?;
    }
    for dep in req.deps_to_remove.iter() {
        dependency::Entity::find()
            .filter(
                Condition::all()
                    .add(dependency::Column::TaskId.eq(req.task_id))
                    .add(dependency::Column::DependsOnId.eq(*dep)),
            )
            .one(db)
            .await
            .map_err(|e| ErrorInternalServerError(format!("couldn't fetch dependancy: {}", e)))?
            .ok_or("dependency couldn't be found")
            .map_err(ErrorInternalServerError)?
            .delete(db)
            .await
            .map_err(|e| ErrorInternalServerError(format!("couldn't delete dependancy: {}", e)))?;
    }
    /*for _script in req.scripts_to_add.iter() {
        //TODO: implement scripts
    }
    for _script in req.scripts_to_remove.iter() {
        //TODO: implement scripts
    }*/

    Ok(req.task_id)
}
#[put("/task")]
async fn update_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<UpdateTaskRequest>,
) -> Result<web::Json<UpdateTaskResponse>> {
    let id = update_task(&data, &req).await?;
    Ok(web::Json(UpdateTaskResponse {
        task_id: id,
        req_id: req.req_id,
    }))
}
#[put("/tasks")]
async fn update_tasks_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<UpdateTasksRequest>,
) -> Result<web::Json<UpdateTasksResponse>> {
    let mut res: UpdateTasksResponse = Vec::new();
    for taskreq in req.iter() {
        let id = update_task(&data, taskreq).await.unwrap_or(-1).to_owned();
        res.push(UpdateTaskResponse {
            task_id: id,
            req_id: taskreq.req_id,
        });
    }
    Ok(web::Json(res))
}

async fn delete_task(
    db: &DatabaseConnection,
    req: &DeleteTaskRequest,
) -> Result<web::Json<DeleteTaskResponse>> {
    task::Entity::find_by_id(req.task_id)
        .one(db)
        .await
        .map_err(|e| ErrorInternalServerError(format!("couldn't find task: {}", e)))?
        .ok_or("couldn't find task by id")
        .map_err(ErrorInternalServerError)?
        .delete(db)
        .await
        .map_err(|e| ErrorInternalServerError(format!("couldn't delete task: {}", e)))?;

    Ok(web::Json(req.req_id))
}

#[delete("/task")]
async fn delete_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<DeleteTaskRequest>,
) -> Result<web::Json<DeleteTaskResponse>> {
    info!("delete_task_request: {:?}", req);
    delete_task(&data, &req).await
}
#[delete("/tasks")]
async fn delete_tasks_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<DeleteTasksRequest>,
) -> Result<web::Json<DeleteTasksResponse>> {
    let mut res: Vec<i32> = vec![];
    for task in req.iter() {
        delete_task(&data, task).await?;
        res.push(task.req_id);
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

    info!("req: {:?}", req);
    let tasks: Vec<task::Model> = task::Entity::find()
        .all(data.as_ref())
        .await
        .map_err(|e| ErrorInternalServerError(format!("couldn't filter tasks: {}", e)))?;

    info!("returning tasks: {:?}", tasks);
    Ok(web::Json(
        tasks.iter().map(|a| a.id).collect::<FilterResponse>(),
    ))
}

async fn get_property_or_err(
    db: &DatabaseConnection,
    prop: &String,
    task_id: i32,
) -> Result<Option<TaskPropVariant>, ()> {
    let typ = task_property::Entity::find()
        .filter(
            Condition::all()
                .add(task_property::Column::TaskId.eq(task_id))
                .add(task_property::Column::Name.eq(prop)),
        )
        .one(db)
        .await
        .map_err(|_| ())?
        .ok_or(())?
        .typ;

    let res = match typ.as_str() {
        "string" => TaskPropVariant::String(
            task_string_property::Entity::find()
                .filter(
                    Condition::all()
                        .add(task_string_property::Column::TaskId.eq(task_id))
                        .add(task_string_property::Column::Name.eq(prop)),
                )
                .one(db)
                .await
                .map_err(|_| ())?
                .ok_or(())?
                .value,
        ),
        "number" => TaskPropVariant::Number(
            task_num_property::Entity::find()
                .filter(
                    Condition::all()
                        .add(task_num_property::Column::TaskId.eq(task_id))
                        .add(task_num_property::Column::Name.eq(prop)),
                )
                .one(db)
                .await
                .map_err(|_| ())?
                .ok_or(())?
                .value
                .clone(),
        ),
        "date" => TaskPropVariant::Date(
            task_date_property::Entity::find()
                .filter(
                    Condition::all()
                        .add(task_date_property::Column::TaskId.eq(task_id))
                        .add(task_date_property::Column::Name.eq(prop)),
                )
                .one(db)
                .await
                .map_err(|_| ())?
                .ok_or(())?
                .value,
        ),
        "boolean" => TaskPropVariant::Boolean(
            task_bool_property::Entity::find()
                .filter(
                    Condition::all()
                        .add(task_bool_property::Column::TaskId.eq(task_id))
                        .add(task_bool_property::Column::Name.eq(prop)),
                )
                .one(db)
                .await
                .map_err(|_| ())?
                .ok_or(())?
                .value,
        ),
        _ => unreachable!(),
    };

    Ok(Some(res))
}

#[get("/prop")]
async fn get_property_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<PropertyRequest>,
) -> Result<web::Json<PropertyResponse>> {
    let mut res = PropertyResponse {
        res: vec![],
        req_id: req.req_id,
    };
    for prop_name in req.properties.iter() {
        let prop = get_property_or_err(data.as_ref(), prop_name, req.task_id)
            .await
            .unwrap_or(None);
        res.res.push(TaskPropOption {
            name: prop_name.to_owned(),
            value: prop,
        });
    }

    Ok(web::Json(res))
}
#[get("/props")]
async fn get_properties_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<PropertiesRequest>,
) -> Result<web::Json<PropertiesResponse>> {
    let mut res = PropertiesResponse {
        res: vec![],
        req_id: req.req_id,
    };
    for prop_name in req.properties.iter() {
        let mut prop_column = TaskPropColumn {
            name: prop_name.to_owned(),
            values: vec![],
        };
        for task_id in req.task_ids.iter() {
            let prop = get_property_or_err(data.as_ref(), prop_name, *task_id)
                .await
                .unwrap_or(None);
            prop_column.values.push(prop);
        }

        res.res.push(prop_column);
    }

    Ok(web::Json(res))
}

#[cfg(test)]
#[path = "./tests/test_create.rs"]
mod test_create;
#[cfg(test)]
#[path = "./tests/test_delete.rs"]
mod test_delete;
#[cfg(test)]
#[path = "./tests/test_props.rs"]
mod test_props;
#[cfg(test)]
#[path = "./tests/test_update.rs"]
mod test_update;
