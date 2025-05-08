use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

use std::env;
use dotenv::dotenv;

use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

type TodoList = Mutex<Vec<Todo>>;

#[utoipa::path(
    get,
    path = "/todos",
    responses(
        (status = 200, description = "List all todos", body = [Todo])
    )
)]
async fn get_todos(data: web::Data<TodoList>) -> impl Responder {
    let todos = data.lock().unwrap();
    HttpResponse::Ok().json(&*todos)
}

#[derive(Deserialize, ToSchema)]
struct CreateTodo {
    title: String,
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodo,
    responses(
        (status = 200, description = "Create a new todo", body = Todo)
    )
)]
async fn add_todo(data: web::Data<TodoList>, new: web::Json<CreateTodo>) -> impl Responder {
    let mut todos = data.lock().unwrap();
    let todo = Todo {
        id: Uuid::new_v4().to_string(),
        title: new.title.clone(),
        completed: false,
    };
    todos.push(todo.clone());
    HttpResponse::Ok().json(todo)
}

#[derive(OpenApi)]
#[openapi(paths(get_todos, add_todo), components(schemas(Todo, CreateTodo)))]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let todos = web::Data::new(Mutex::new(Vec::<Todo>::new()));

    dotenv().ok();

    // Check if we have the PORT environment variable set (for Render or Docker)
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()); // fallback to 8080 for local
    let address = format!("0.0.0.0:{}", port);

    println!("Starting server on {}", address);

    HttpServer::new(move || {
        App::new()
            .app_data(todos.clone())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .route("/todos", web::get().to(get_todos))
            .route("/todos", web::post().to(add_todo))
    })
    .bind(address)?
    .run()
    .await
}
