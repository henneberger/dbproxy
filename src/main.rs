#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[macro_use]
extern crate serde_derive;
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Validation, Header, Algorithm};
use actix::prelude::*;
use actix_http::{HttpService, KeepAlive};
use actix_service::map_config;
use actix_web::dev::{AppConfig, Body, Server};
use actix_web::http::{header::CONTENT_TYPE, HeaderValue, StatusCode};
use actix_web::{web, App, Error, HttpRequest, HttpResponse};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use uuid::Uuid;
use crate::utils::Writer;
#[derive(Serialize, Deserialize)]
struct TodoClaim {
    todo_id: Uuid
}
#[derive(Serialize, Deserialize)]
struct UserClaim {
    user_id: Uuid
}
#[derive(Serialize, Deserialize)]
struct TodoIdClaim {
    user_id: Uuid,
    todo_id: Uuid,
}
#[derive(Serialize, Deserialize)]
struct TodoByIdInfo {
    id: String,
}
#[derive(Serialize, Deserialize)]
struct CreateTodoInfo {
    title: String,
    completed: bool,
}
#[derive(Serialize, Deserialize)]
struct TodosInfo {
}
#[derive(Serialize, Deserialize)]
struct UpdateTodoInfo {
    id: String,
    title: String,
    completed: bool,
}
#[derive(Serialize, Deserialize)]
struct ClearCompletedInfo {
    completed: bool,
}


#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct TodoResp {
    pub id: String,
    pub title: String,
    pub completed: bool,
}


//todo: fix when https://github.com/Keats/jsonwebtoken/issues/120 is closed
struct AppState<'tok> {
    key: DecodingKey<'tok>,
    encode: EncodingKey,
    val: Validation
}
mod db_pg;
mod models;
mod utils;
use models::*;
use crate::db_pg::*;
use bytes::{BytesMut};

async fn create<'tok>(
    req: HttpRequest,
    db: web::Data<Addr<PgConnection>>,
    app_state: web::Data<AppState<'tok>>,
    query: web::Query<CreateTodoInfo>,
) -> Result<HttpResponse, Error> {
    let bearer = get_bearer(&req);
    let auth_token = decode::<UserClaim>(bearer.to_owned().as_str(), &app_state.key, &app_state.val)
        .unwrap();

    let todo = db
        .send(CreateTodo(auth_token.claims.user_id, Uuid::new_v4(), query.title.to_owned(), query.completed));

    let rslt = todo.await;

    let todo = rslt.unwrap().unwrap();

    let result = TodoResp {
        id: sign_todo_id(&app_state, todo.user_id, todo.todo_id),
        title: todo.title,
        completed: todo.completed
    };

    let mut body = BytesMut::with_capacity(200);
    serde_json::to_writer(
        Writer(&mut body),
        &result,
    )
    .unwrap();

    let mut res = HttpResponse::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(res)
}

async fn todo_by_id<'tok>(
    req: HttpRequest,
    db: web::Data<Addr<PgConnection>>,
    app_state: web::Data<AppState<'tok>>,
    query: web::Query<TodoByIdInfo>,
) -> Result<HttpResponse, Error> {
    let bearer = get_bearer(&req);
    let auth_token = decode::<UserClaim>(bearer.to_owned().as_str(), &app_state.key, &app_state.val)
        .unwrap();

    let todo_id_claim = decode::<TodoIdClaim>(query.id.to_owned().as_str(), &app_state.key, &app_state.val)
    .unwrap();

    let todo = db
        .send(TodoById(auth_token.claims.user_id, todo_id_claim.claims.todo_id))
        .await
        .unwrap()
        .unwrap();

    let result = TodoResp {
        id: sign_todo_id(&app_state, todo.user_id, todo.todo_id),
        title: todo.title,
        completed: todo.completed,
    };

    let mut body = BytesMut::with_capacity(200);
    serde_json::to_writer(
        Writer(&mut body),
        &result,
    )
    .unwrap();

    let mut res = HttpResponse::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(res)
}

async fn todos<'tok>(
    req: HttpRequest,
    db: web::Data<Addr<PgConnection>>,
    app_state: web::Data<AppState<'tok>>,
    query: web::Query<TodosInfo>,
) -> Result<HttpResponse, Error> {
    let bearer = get_bearer(&req);
    let auth_token = decode::<UserClaim>(bearer.to_owned().as_str(), &app_state.key, &app_state.val)
        .unwrap();

    let todos: Vec<Todo> = db
        .send(Todos(auth_token.claims.user_id))
        .await
        .unwrap()
        .unwrap();

    let result: Vec<TodoResp> = todos.iter()
    .map({|t| {
        TodoResp {
            id: sign_todo_id(&app_state, t.user_id, t.todo_id),
            title: t.title.to_owned(),
            completed: t.completed,
        }
    }})
    .collect();

    let mut body = BytesMut::with_capacity(200);
    serde_json::to_writer(
        Writer(&mut body),
        &result,
    )
    .unwrap();

    let mut res = HttpResponse::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(res)
}

async fn update_todo<'tok>(
    req: HttpRequest,
    db: web::Data<Addr<PgConnection>>,
    app_state: web::Data<AppState<'tok>>,
    query: web::Query<UpdateTodoInfo>,
) -> Result<HttpResponse, Error> {
    let bearer = get_bearer(&req);
    let auth_token = decode::<UserClaim>(bearer.to_owned().as_str(), &app_state.key, &app_state.val)
        .unwrap();

    let todo_id_claim = decode::<TodoIdClaim>(query.id.to_owned().as_str(), &app_state.key, &app_state.val)
        .unwrap();

    let todo = db
        .send(UpdateTodo(todo_id_claim.claims.user_id, todo_id_claim.claims.todo_id, query.title.to_owned(), query.completed))
        .await
        .unwrap()
        .unwrap();
    let result = TodoResp {
        id: sign_todo_id(&app_state, todo.user_id, todo.todo_id),
        title: todo.title,
        completed: todo.completed,
    };

    let mut body = BytesMut::with_capacity(200);
    serde_json::to_writer(
        Writer(&mut body),
        &result,
    )
    .unwrap();

    let mut res = HttpResponse::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(res)
}

async fn clear_completed<'tok>(
    req: HttpRequest,
    db: web::Data<Addr<PgConnection>>,
    app_state: web::Data<AppState<'tok>>,
    query: web::Query<ClearCompletedInfo>,
) -> Result<HttpResponse, Error> {
    let bearer = get_bearer(&req);
    let auth_token = decode::<UserClaim>(bearer.to_owned().as_str(), &app_state.key, &app_state.val)
        .unwrap();

    let todo = db
        .send(ClearCompleted(auth_token.claims.user_id, query.completed))
        .await
        .unwrap()
        .unwrap();

    let mut body = BytesMut::with_capacity(200);
    serde_json::to_writer(
        Writer(&mut body),
        &todo,
    )
    .unwrap();

    let mut res = HttpResponse::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(res)
}

fn sign_todo_id<'tok>(app_state: &web::Data<AppState<'tok>>, user_id: Uuid, todo_id: Uuid) -> String {
    let token = encode(&Header::default(), &TodoIdClaim {user_id, todo_id}, &app_state.encode)
    .unwrap();

    token
}
fn get_bearer(req: &HttpRequest) -> String {
    let auth = req.headers().get("Authorization");
    let b = String::from(auth.unwrap().to_str().unwrap());
    let bearer = if let Some(pos) = b.find("Bearer") {
        b.split_at(pos + 7).1.parse::<String>().ok().unwrap()
    } else {
        String::new()
    };

    bearer
}

fn ssl_acceptor() -> SslAcceptor {
    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file("cert.pem")
        .unwrap();
    builder.set_alpn_select_callback(|_, protos| {
        const H2: &[u8] = b"\x02h2";
        if protos.windows(3).any(|window| window == H2) {
            Ok(b"h2")
        } else {
            Err(openssl::ssl::AlpnError::NOACK)
        }
    });
    builder.set_alpn_protos(b"\x02h2").unwrap();
    builder.build()
}
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Started http server: 127.0.0.1:8080");
    const DB_URL: &str =
        "postgres://benchmarkdbuser:benchmarkdbpass@tfb-database/hello_world";
    Server::build()
    .backlog(1024)
    .bind("techempower", "0.0.0.0:8080", move || {
        HttpService::build()
            .keep_alive(KeepAlive::Os)
            .client_timeout(0)
            .h2(map_config(
                App::new()
                    .data(AppState {
                        key: DecodingKey::from_secret(b"abc"), 
                        encode: EncodingKey::from_secret(b"abc"), 
                        val: Validation {
                            leeway: 0,
                    
                            validate_exp: false,
                            validate_nbf: false,
                    
                            iss: None,
                            sub: None,
                            aud: None,
                    
                            algorithms: vec![Algorithm::HS256],
                        }
                    })
                    .data_factory(|| PgConnection::connect(DB_URL))
                    .service(web::resource("/create").to(create))
                    .service(web::resource("/todoById").to(todo_by_id))
                    .service(web::resource("/todos").to(todos))
                    .service(web::resource("/updateTodo").to(update_todo))
                    .service(web::resource("/clearCompleted").to(clear_completed)),
                |_| AppConfig::default(),
            ))
            .openssl(ssl_acceptor())
    })?
    .start()
    .await
}

