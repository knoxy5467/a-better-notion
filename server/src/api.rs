use std::{fs, io::Write, path::Path};

use crate::database::*;
use actix_settings::BasicSettings;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
#[allow(unused)]
use actix_web::{delete, get, post, put, web, Responder, Result};
use common::{
    backend::*, Comparator, Filter, Operator, PrimitiveField, TaskID, TaskPropVariant, ViewData,
};
use log::info;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use sea_orm::{
    entity::prelude::*, ActiveValue::NotSet, Condition, IntoActiveModel, QuerySelect, Set,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub database_url: String,
}
type AbnSettings = BasicSettings<DatabaseSettings>;

pub fn load_settings() -> Result<AbnSettings, actix_settings::Error> {
    // if Server.toml does not exist in working directory:
    let settings_filepath = Path::new(".abn_settings").join("Server.toml");
    match fs::metadata(&settings_filepath) {
        Ok(_) => {
            return AbnSettings::parse_toml(&settings_filepath);
        },
        Err(_) => {
            println!("creating directory");
            fs::create_dir(Path::new(".abn_settings")).unwrap();
            let mut settings = AbnSettings::parse_toml(&settings_filepath);
            // write database url to the file
            // Open a file with append option
            let mut settings_file = fs::OpenOptions::new()
                .append(true)
                .write(true)
                .open(&settings_filepath)
                .expect("cannot open file");

            // Write db id to a file
            settings_file
                .write("\ndatabase_url = \"postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask\"".as_bytes())
                .expect("write failed");
            //fs::write(&settings_filepath, "postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask").unwrap();
            
            return AbnSettings::parse_toml(&settings_filepath);
        },
    }
}

/// get /task endpoint for retrieving a single TaskShort
#[get("/task")]
async fn get_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<ReadTaskShortRequest>,
) -> Result<impl Responder> {
    info!("get_task_request, req: {:?}", req);
    let db = data;
    let task = task::Entity::find_by_id(req.task_id)
        .one(db.as_ref())
        .await
        .map_err(|e| ErrorInternalServerError(format!("SQL error: {}", e)))?; // TODO handle this error better, if it does not exist then it should be a http 204 error
    info!("get_task_request, found_task: {:?}", task);
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
    info!("get_tasks_request, req: {:?}", req);
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
    info!("finished get_tasks_request, res: {:?}", res);

    Ok(web::Json(res))
}

/// post /task endpoint creates a single task
pub async fn create_task(db: &DatabaseConnection, req: &CreateTaskRequest) -> Result<TaskID> {
    let task_model = task::ActiveModel {
        id: NotSet,
        title: Set(req.name.clone()),
        completed: Set(req.completed),
        last_edited: Set(chrono::Local::now().naive_local()),
    };
    let result_task = task::Entity::insert(task_model)
        .exec(db)
        .await
        .map_err(|e| ErrorInternalServerError(format!("task not inserted: {}", e)))?; //TODO handle this error better, for example for unique constraint violation
    info!("create_task, result_task: {:?}", result_task);
    Ok(result_task.last_insert_id)
}

#[post("/task")]
async fn create_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<CreateTaskRequest>,
) -> Result<web::Json<CreateTaskResponse>> {
    info!("create_task_request, req: {:?}", req);
    let id = create_task(&data, &req).await?;
    info!("created task with id: {:?}", id);
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
    info!("create_tasks_request, req: {:?}", req);
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
pub async fn update_task(db: &DatabaseConnection, req: &UpdateTaskRequest) -> Result<TaskID> {
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
    if req.checked.is_some() || req.name.is_some() {
        task.update(db).await.map_err(ErrorInternalServerError)?;
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
                                .add(
                                    task_string_property::Column::TaskPropertyName
                                        .eq(prop.name.to_owned()),
                                ),
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
                                .add(
                                    task_date_property::Column::TaskPropertyName
                                        .eq(prop.name.to_owned()),
                                ),
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
                                .add(
                                    task_num_property::Column::TaskPropertyName
                                        .eq(prop.name.to_owned()),
                                ),
                        )
                        .one(db)
                        .await
                        .map_err(|e| {
                            ErrorInternalServerError(format!("couldn't fetch property: {}", e))
                        })?
                        .ok_or("couldn't find property by name")
                        .map_err(ErrorInternalServerError)?;
                    let mut p = p.into_active_model();
                    p.value = Set(Decimal::from_f64(*val).unwrap());
                }
                TaskPropVariant::Boolean(val) => {
                    let p = task_bool_property::Entity::find()
                        .filter(
                            Condition::all()
                                .add(task_bool_property::Column::TaskId.eq(req.task_id))
                                .add(
                                    task_bool_property::Column::TaskPropertyName
                                        .eq(prop.name.to_owned()),
                                ),
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
                    task_property_name: Set(prop.name.to_owned()),
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
                    task_property_name: Set(prop.name.to_owned()),
                    value: Set(Decimal::from_f64(*val).unwrap()),
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
                    task_property_name: Set(prop.name.to_owned()),
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
                    task_property_name: Set(prop.name.to_owned()),
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
                    .add(task_property::Column::Name.eq(prop)),
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

    info!("update_task, updated task: {:?}", req.task_id);
    Ok(req.task_id)
}
#[put("/task")]
async fn update_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<UpdateTaskRequest>,
) -> Result<web::Json<UpdateTaskResponse>> {
    info!("update_task_request, req: {:?}", req);
    let id = update_task(&data, &req).await?;
    info!("update_task_request, completed id : {:?}", id);
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
    info!("update_tasks_request, req: {:?}", req);
    let mut res: UpdateTasksResponse = Vec::new();
    for taskreq in req.iter() {
        let id = update_task(&data, taskreq).await.unwrap_or(-1).to_owned();
        res.push(UpdateTaskResponse {
            task_id: id,
            req_id: taskreq.req_id,
        });
    }
    info!("update_tasks_request, completed res: {:?}", res);
    Ok(web::Json(res))
}

async fn delete_task(
    db: &DatabaseConnection,
    req: &DeleteTaskRequest,
) -> Result<web::Json<DeleteTaskResponse>> {
    info!("delete_task, req: {:?}", req);
    task::Entity::find_by_id(req.task_id)
        .one(db)
        .await
        .map_err(|e| ErrorInternalServerError(format!("couldn't find task: {}", e)))?
        .ok_or("couldn't find task by id")
        .map_err(ErrorInternalServerError)?
        .delete(db)
        .await
        .map_err(|e| ErrorInternalServerError(format!("couldn't delete task: {}", e)))?;
    info!("delete_task, deleted task: {:?}", req.task_id);
    Ok(web::Json(req.req_id))
}

#[delete("/task")]
async fn delete_task_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<DeleteTaskRequest>,
) -> Result<web::Json<DeleteTaskResponse>> {
    info!("delete_task_request, req: {:?}", req);
    delete_task(&data, &req).await
}
#[delete("/tasks")]
async fn delete_tasks_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<DeleteTasksRequest>,
) -> Result<web::Json<DeleteTasksResponse>> {
    info!("delete_tasks_request, req: {:?}", req);
    let mut res: Vec<u64> = vec![];
    for task in req.iter() {
        delete_task(&data, task).await?;
        res.push(task.req_id);
    }
    info!("delete_tasks_request, completed res: {:?}", res);
    Ok(web::Json(res))
}

fn construct_filter(filter: &Filter) -> actix_web::Result<Condition> {
    match filter {
        Filter::Leaf {
            field,
            comparator,
            immediate,
        } => {
            let mut condition = Condition::all();
            match immediate {
                TaskPropVariant::Number(imm) => {
                    condition =
                        condition.add(task_num_property::Column::TaskPropertyName.eq(field));
                    condition = match comparator {
                        Comparator::LT => condition.add(task_num_property::Column::Value.lt(*imm)),
                        Comparator::LEQ => {
                            condition.add(task_num_property::Column::Value.lte(*imm))
                        }
                        Comparator::GT => condition.add(task_num_property::Column::Value.gt(*imm)),
                        Comparator::GEQ => {
                            condition.add(task_num_property::Column::Value.gte(*imm))
                        }
                        Comparator::EQ => condition.add(task_num_property::Column::Value.eq(*imm)),
                        Comparator::NEQ => condition.add(task_num_property::Column::Value.ne(*imm)),
                        _ => {
                            return Err(actix_web::error::ErrorInternalServerError(format!(
                                "Invalid comparator {:?} for type number",
                                {}
                            )))
                        }
                    };
                }
                TaskPropVariant::Date(imm) => {
                    condition =
                        condition.add(task_date_property::Column::TaskPropertyName.eq(field));
                    condition = match comparator {
                        Comparator::LT => condition.add(task_date_property::Column::Value.lt(*imm)),
                        Comparator::LEQ => {
                            condition.add(task_date_property::Column::Value.lte(*imm))
                        }
                        Comparator::GT => condition.add(task_date_property::Column::Value.gt(*imm)),
                        Comparator::GEQ => {
                            condition.add(task_date_property::Column::Value.gte(*imm))
                        }
                        Comparator::EQ => condition.add(task_date_property::Column::Value.eq(*imm)),
                        Comparator::NEQ => {
                            condition.add(task_date_property::Column::Value.ne(*imm))
                        }
                        _ => {
                            return Err(actix_web::error::ErrorInternalServerError(format!(
                                "Invalid comparator {:?} for type date",
                                {}
                            )))
                        }
                    };
                }
                TaskPropVariant::Boolean(imm) => {
                    condition =
                        condition.add(task_bool_property::Column::TaskPropertyName.eq(field));
                    condition = match comparator {
                        Comparator::EQ => condition.add(task_bool_property::Column::Value.eq(*imm)),
                        Comparator::NEQ => {
                            condition.add(task_bool_property::Column::Value.ne(*imm))
                        }
                        _ => {
                            return Err(actix_web::error::ErrorInternalServerError(format!(
                                "Invalid comparator {:?} for type boolean",
                                {}
                            )))
                        }
                    }
                }
                TaskPropVariant::String(imm) => {
                    condition =
                        condition.add(task_string_property::Column::TaskPropertyName.eq(field));
                    condition = match comparator {
                        Comparator::LT => {
                            condition.add(task_string_property::Column::Value.lt(imm.clone()))
                        }
                        Comparator::LEQ => {
                            condition.add(task_string_property::Column::Value.lte(imm.clone()))
                        }
                        Comparator::GT => {
                            condition.add(task_string_property::Column::Value.gt(imm.clone()))
                        }
                        Comparator::GEQ => {
                            condition.add(task_string_property::Column::Value.gte(imm.clone()))
                        }
                        Comparator::EQ => {
                            condition.add(task_string_property::Column::Value.eq(imm.clone()))
                        }
                        Comparator::NEQ => {
                            condition.add(task_string_property::Column::Value.ne(imm.clone()))
                        }
                        Comparator::CONTAINS => condition
                            .add(task_string_property::Column::Value.like(format!("%{}%", imm))),
                        Comparator::NOTCONTAINS => {
                            condition.add(Condition::not(Condition::all().add(
                                task_string_property::Column::Value.like(format!("%{}%", imm)),
                            )))
                        }
                        Comparator::LIKE => {
                            condition.add(task_string_property::Column::Value.like(imm.clone()))
                        }
                    }
                }
            };
            Ok(condition)
        }
        Filter::LeafPrimitive {
            field,
            comparator,
            immediate,
        } => match field {
            PrimitiveField::TITLE => {
                let mut condition = Condition::all();
                let imm = match immediate {
                    TaskPropVariant::String(a) => a,
                    _ => return Err(ErrorInternalServerError("die")),
                };

                condition = match comparator {
                    Comparator::LT => condition.add(task::Column::Title.lt(imm.clone())),
                    Comparator::LEQ => condition.add(task::Column::Title.lte(imm.clone())),
                    Comparator::GT => condition.add(task::Column::Title.gt(imm.clone())),
                    Comparator::GEQ => condition.add(task::Column::Title.gte(imm.clone())),
                    Comparator::EQ => condition.add(task::Column::Title.eq(imm.clone())),
                    Comparator::NEQ => condition.add(task::Column::Title.ne(imm.clone())),
                    Comparator::CONTAINS => {
                        condition.add(task::Column::Title.like(format!("%{}%", imm)))
                    }
                    Comparator::NOTCONTAINS => condition.add(Condition::not(
                        Condition::all().add(task::Column::Title.like(format!("%{}%", imm))),
                    )),
                    Comparator::LIKE => condition.add(task::Column::Title.like(imm.clone())),
                };
                Ok(condition)
            }
            PrimitiveField::COMPLETED => {
                let mut condition = Condition::all();
                let imm = match immediate {
                    TaskPropVariant::Boolean(a) => a,
                    _ => return Err(ErrorInternalServerError("invalid type")),
                };

                condition = match comparator {
                    Comparator::EQ => condition.add(task::Column::Completed.eq(*imm)),
                    Comparator::NEQ => condition.add(task::Column::Completed.ne(*imm)),
                    _ => return Err(ErrorInternalServerError("invalid comparator")),
                };
                Ok(condition)
            }
            PrimitiveField::LASTEDITED => {
                let mut condition = Condition::all();
                let imm = match immediate {
                    TaskPropVariant::Date(a) => a,
                    _ => return Err(ErrorInternalServerError("invalid type")),
                };

                condition = match comparator {
                    Comparator::LT => condition.add(task::Column::LastEdited.lt(*imm)),
                    Comparator::LEQ => condition.add(task::Column::LastEdited.lte(*imm)),
                    Comparator::GT => condition.add(task::Column::LastEdited.gt(*imm)),
                    Comparator::GEQ => condition.add(task::Column::LastEdited.gte(*imm)),
                    Comparator::EQ => condition.add(task::Column::LastEdited.eq(*imm)),
                    Comparator::NEQ => condition.add(task::Column::LastEdited.ne(*imm)),
                    _ => return Err(ErrorInternalServerError("invalid comparator")),
                };
                Ok(condition)
            }
        },
        Filter::Operator { op, childs } => {
            if let Operator::NOT = op {
                match construct_filter(&childs[0]) {
                    Ok(filter) => return Ok(Condition::not(filter)),
                    Err(err) => return Err(err),
                }
            }
            let mut condition = match op {
                Operator::AND => Condition::all(),
                Operator::OR => Condition::any(),
                _ => unreachable!(),
            };
            for child in childs.iter() {
                match construct_filter(child) {
                    Ok(filter) => condition = condition.add(filter),
                    Err(err) => return Err(err),
                }
            }
            Ok(condition)
        }
        Filter::None => Ok(Condition::any()),
    }
}

pub async fn filter(
    db: &DatabaseConnection,
    req: &FilterRequest,
) -> Result<web::Json<FilterResponse>> {
    if let Filter::None = req.filter {
        let tasks = task::Entity::find()
            .all(db)
            .await
            .map_err(ErrorInternalServerError)?
            .iter()
            .map(|a| a.id)
            .collect();
        return Ok(web::Json(FilterResponse {
            tasks,
            req_id: req.req_id,
        }));
    }

    let filter = construct_filter(&req.filter)?;

    let tasks: Vec<task::Model> = task::Entity::find()
        .join(
            sea_orm::JoinType::LeftJoin,
            task::Relation::TaskNumProperty.def(),
        )
        .join(
            sea_orm::JoinType::LeftJoin,
            task::Relation::TaskBoolProperty.def(),
        )
        .join(
            sea_orm::JoinType::LeftJoin,
            task::Relation::TaskStringProperty.def(),
        )
        .join(
            sea_orm::JoinType::LeftJoin,
            task::Relation::TaskDateProperty.def(),
        )
        .filter(filter)
        .all(db)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("couldn't filter tasks: {}", e))
        })?;

    Ok(web::Json(FilterResponse {
        tasks: tasks.iter().map(|a| a.id).collect::<Vec<i32>>(),
        req_id: req.req_id,
    }))
}

/// get /filter endpoint for retrieving some number of TaskShorts
#[get("/filter")]
async fn get_filter_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<FilterRequest>,
) -> Result<impl Responder> {
    filter(&data, &req).await
}

async fn get_property_or_err(
    db: &DatabaseConnection,
    prop: &String,
    task_id: i32,
) -> Result<Option<TaskPropVariant>, ()> {
    info!(
        "get_property_or_err, prop: {:?}, task_id: {:?}",
        prop, task_id
    );
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
                        .add(task_string_property::Column::TaskPropertyName.eq(prop)),
                )
                .one(db)
                .await
                .map_err(|_| ())?
                .ok_or(())?
                .value,
        ),
        "number" => TaskPropVariant::Number(
            Decimal::to_f64(
                &task_num_property::Entity::find()
                    .filter(
                        Condition::all()
                            .add(task_num_property::Column::TaskId.eq(task_id))
                            .add(task_num_property::Column::TaskPropertyName.eq(prop)),
                    )
                    .one(db)
                    .await
                    .map_err(|_| ())?
                    .ok_or(())?
                    .value,
            )
            .unwrap(),
        ),
        "date" => TaskPropVariant::Date(
            task_date_property::Entity::find()
                .filter(
                    Condition::all()
                        .add(task_date_property::Column::TaskId.eq(task_id))
                        .add(task_date_property::Column::TaskPropertyName.eq(prop)),
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
                        .add(task_bool_property::Column::TaskPropertyName.eq(prop)),
                )
                .one(db)
                .await
                .map_err(|_| ())?
                .ok_or(())?
                .value,
        ),
        _ => unreachable!(),
    };

    info!("get_property_or_err, res: {:?}", res);
    Ok(Some(res))
}

#[get("/prop")]
async fn get_property_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<PropertyRequest>,
) -> Result<web::Json<PropertyResponse>> {
    info!("get_property_request, req: {:?}", req);
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
    info!("get_property_request, res: {:?}", res);
    Ok(web::Json(res))
}
#[get("/props")]
async fn get_properties_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<PropertiesRequest>,
) -> Result<web::Json<PropertiesResponse>> {
    info!("get_properties_request, req: {:?}", req);
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

    info!("get_properties_request, res: {:?}", res);
    Ok(web::Json(res))
}

#[get("/views")]
async fn get_views_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<GetViewRequest>,
) -> Result<web::Json<GetViewResponse>> {
    let views = view::Entity::find()
        .all(data.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(web::Json(GetViewResponse {
        req_id: req.to_owned(),
        views: views
            .iter()
            .map(|view| ViewData {
                name: view.name.clone(),
                view_id: view.id,
                filter: serde_json::from_str(&view.filter).unwrap(),
                props: view.properties.clone(),
            })
            .collect(),
    }))
}
#[post("/view")]
async fn create_view_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<CreateViewRequest>,
) -> Result<web::Json<CreateViewResponse>> {
    let view_model = view::ActiveModel {
        id: NotSet,
        name: Set(req.name.clone()),
        properties: Set(req.props.clone()),
        filter: Set(serde_json::to_string(&req.filter).unwrap()),
    };
    let res = view::Entity::insert(view_model)
        .exec(data.as_ref())
        .await
        .map_err(|e| ErrorInternalServerError(format!("view not inserted: {}", e)))?;
    Ok(web::Json(CreateViewResponse {
        view_id: res.last_insert_id,
        req_id: req.req_id,
    }))
}
#[put("/view")]
async fn update_view_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<UpdateViewRequest>,
) -> Result<web::Json<UpdateViewResponse>> {
    let view = view::Entity::find_by_id(req.view.view_id)
        .one(data.as_ref())
        .await
        .map_err(ErrorInternalServerError)?
        .ok_or("couldn't find view by id")
        .map_err(ErrorInternalServerError)?;
    let mut view: view::ActiveModel = view.into();
    view.name = Set(req.view.name.clone());
    view.properties = Set(req.view.props.clone());
    view.filter = Set(serde_json::to_string(&req.view.filter).unwrap());
    view.update(data.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(web::Json(req.req_id))
}

#[delete("/view")]
async fn delete_view_request(
    data: web::Data<DatabaseConnection>,
    req: web::Json<DeleteViewRequest>,
) -> Result<web::Json<DeleteViewResponse>> {
    view::Entity::find_by_id(req.to_owned())
        .one(data.as_ref())
        .await
        .map_err(ErrorInternalServerError)?
        .ok_or("couldn't find view by id")
        .map_err(ErrorInternalServerError)?
        .delete(data.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(web::Json(()))
}

#[cfg(test)]
#[path = "./tests/test_create.rs"]
mod test_create;
#[cfg(test)]
#[path = "./tests/test_delete.rs"]
mod test_delete;
#[cfg(test)]
#[path = "./tests/test_filter.rs"]
mod test_filter;
#[cfg(test)]
#[path = "./tests/test_props.rs"]
mod test_props;
#[cfg(test)]
#[path = "./tests/test_update.rs"]
mod test_update;
#[cfg(test)]
#[path = "./tests/test_views.rs"]
mod test_views;
