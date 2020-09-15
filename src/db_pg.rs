use std::io;

use actix::prelude::*;
use futures::FutureExt;
use tokio_postgres::{connect, Client, NoTls, Statement};
use uuid::Uuid;
use tokio_postgres::{Row};
use crate::models::{Todo, UpdateTodoSuccess};

/// Postgres interface
pub struct PgConnection {
    cl: Client,
    create_todo: Statement,
    todo_by_id: Statement,
    todos: Statement,
    update_todo: Statement,
    clear_completed: Statement,
}

impl Actor for PgConnection {
    type Context = Context<Self>;
}

impl PgConnection {
    pub async fn connect(db_url: &str) -> Result<Addr<PgConnection>, io::Error> {
        let (cl, conn) = connect(db_url, NoTls)
            .await
            .expect("can not connect to postgresql");
        actix_rt::spawn(conn.map(|_| ()));
        let create_todo = cl.prepare("INSERT into todo(user_id, todo_id, title, completed) values ($1, $2, $3, $4)").await.unwrap();
        let todo_by_id = cl.prepare("SELECT user_id, todo_id, title, completed FROM todo WHERE user_id=$1 AND todo_id=$2").await.unwrap();
        let todos = cl.prepare("SELECT user_id, todo_id, title, completed FROM todo WHERE user_id=$1").await.unwrap();
        let update_todo = cl.prepare("UPDATE todo SET title=$3, completed=$4 WHERE user_id = $1 AND todo_id=$2").await.unwrap();
        let clear_completed = cl.prepare("DELETE FROM todo WHERE user_id=$1 AND completed=$2").await.unwrap();

        println!("connecting");
        Ok(PgConnection::create(move |_| PgConnection {
            cl,
            create_todo,
            todo_by_id,
            todos,
            update_todo,
            clear_completed,
        }))
    }
}

pub struct CreateTodo(pub Uuid, pub Uuid, pub String, pub bool);

impl Message for CreateTodo {
    type Result = io::Result<Todo>;
}

impl Handler<CreateTodo> for PgConnection {
    type Result = ResponseFuture<Result<Todo, io::Error>>;

    fn handle(&mut self, msg: CreateTodo, _: &mut Self::Context) -> Self::Result {
        let fut = self.cl.query_raw(&self.create_todo, &[&msg.0, &msg.1, &msg.2, &msg.3]);
        Box::pin( async move {
            let res = fut.await;
            Ok(Todo{
                user_id: msg.0,
                todo_id: msg.1,
                title: msg.2,
                completed: msg.3,
            })
        })
    }
}

pub struct TodoById(pub Uuid, pub Uuid);

impl Message for TodoById {
    type Result = io::Result<Todo>;
}

impl Handler<TodoById> for PgConnection {
    type Result = ResponseFuture<Result<Todo, io::Error>>;

    fn handle(&mut self, msg: TodoById, _: &mut Self::Context) -> Self::Result {
        let fut = self.cl.query_one(&self.todo_by_id, &[&msg.0, &msg.1]);
        Box::pin(async move {
            let row = fut
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
            let todo = Todo {
                todo_id: row.get(0),
                user_id: row.get(1),
                title: row.get(2),
                completed: row.get(3),
            };

            Ok(todo)
        })
    }
}

pub struct Todos(pub Uuid);

impl Message for Todos {
    type Result = io::Result<Vec<Todo>>;
}

impl Handler<Todos> for PgConnection {
    type Result = ResponseFuture<Result<Vec<Todo>, io::Error>>;

    fn handle(&mut self, msg: Todos, _: &mut Self::Context) -> Self::Result {

        let cl = self.cl.clone();
        // let param = &msg.0.clone();
        let st = self.todos.clone();

        Box::pin(async move {
            let m = cl.query(&st, &[&msg.0])
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

            let m = m.iter()
                .map(|row| {
                    Todo {
                        user_id: row.get(0),
                        todo_id: row.get(1),
                        title: row.get(2),
                        completed: row.get(3),
                    }
                }).collect();
            Ok(m)
        })
    }
}


pub struct UpdateTodo(pub Uuid, pub Uuid, pub String, pub bool);

impl Message for UpdateTodo {
    type Result = io::Result<Todo>;
}

impl Handler<UpdateTodo> for PgConnection {
    type Result = ResponseFuture<Result<Todo, io::Error>>;

    fn handle(&mut self, msg: UpdateTodo, _: &mut Self::Context) -> Self::Result {
        let fut = self.cl.query_raw(&self.update_todo, &[&msg.0, &msg.1, &msg.2, &msg.3]);
        Box::pin( async move {
            let res = fut.await;
            Ok(Todo{
                user_id: msg.0,
                todo_id: msg.1,
                title: msg.2,
                completed: msg.3,
            })
        })
    }
}

pub struct ClearCompleted(pub Uuid, pub bool);

impl Message for ClearCompleted {
    type Result = io::Result<UpdateTodoSuccess>;
}

impl Handler<ClearCompleted> for PgConnection {
    type Result = ResponseFuture<Result<UpdateTodoSuccess, io::Error>>;

    fn handle(&mut self, msg: ClearCompleted, _: &mut Self::Context) -> Self::Result {
        let fut = self.cl.query_raw(&self.clear_completed, &[&msg.0, &msg.1]);
        Box::pin( async move {
            let res = fut.await;
            Ok(UpdateTodoSuccess{
                success: true,
            })
        })
    }
}