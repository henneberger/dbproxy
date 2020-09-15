#![allow(dead_code)]
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message: &'static str,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct Todo {
    pub user_id: Uuid,
    pub todo_id: Uuid,
    pub title: String,
    pub completed: bool,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct UpdateTodoSuccess {
    pub success: bool,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
}