use actix_web::{get, web, Responder, Result};
use common::backend::*;

/// get /task endpoint for retrieving a single TaskShort
#[get("/task")]
async fn get_task_request(req: web::Json<ReadTaskShortRequest>) -> Result<impl Responder> {
    println!("requesting task id {}", req.task_id);

    // do diesel stuff here

    Ok(web::Json(ReadTaskShortResponse {
        task_id: req.task_id,
        name: "heyo".to_string(),
        completed: false,
        props: Vec::new(),
        deps: Vec::new(),
        scripts: Vec::new(),
    }))
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
    }]))
}
