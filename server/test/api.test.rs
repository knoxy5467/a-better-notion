use actix_web::{web::Json};
use backend::*;
use api::*;

#[cfg(test)]
mod tests {
    //TODO: figure out how to actually test this with a real database--
    //maybe add a task, request the taskshort and ensure that works, then
    //delete it later? Or we could just ensure that the params are what
    //they should be?
    #[test]
    fn test_get_task_request() {
        let res = get_task_request(web::Json(ReadTaskShortRequest {
            task_id: 0
        }));

        let expected = Ok(web::Json(ReadTaskShortResponse {
            task_id: 0,
            name: "heyo".to_string(),
            completed: false,
            props: Vec::new(),
            deps: Vec::new(),
            scripts: Vec::new()
        }));

        assert_eq!(res, expected)
    }

}
