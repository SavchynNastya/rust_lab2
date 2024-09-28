#[macro_use] extern crate rocket;

use std::fs::{self, OpenOptions};
use rocket::serde::{Serialize, Deserialize};
use rocket::serde::json::{Json};
use rocket_dyn_templates::{context, Template};
use std::sync::Mutex;
use std::io::{BufReader};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
struct Task {
    id: Option<usize>,
    title: String,
    description: String,
    completed: bool,
}

#[derive(Default)]
struct TodoList {
    tasks: Vec<Task>,
}

fn save_tasks(tasks: &Vec<Task>) {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("tasks.json")
        .unwrap();

    serde_json::to_writer(file, tasks).unwrap();
}

fn load_tasks() -> Vec<Task> {
    let path = Path::new("tasks.json");
    if !path.exists() {
        return vec![];
    }

    let file = fs::File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_else(|_| vec![])
}

#[get("/")]
fn index(todo_list: &rocket::State<Mutex<TodoList>>) -> Template {
    let tasks = todo_list.lock().unwrap().tasks.clone();
    Template::render("index", &context! { tasks })
}

#[post("/add", format = "application/json", data = "<task>")]
fn add_task(task: Json<Task>, todo_list: &rocket::State<Mutex<TodoList>>) -> Json<Task> {
    let mut todo_list = todo_list.lock().unwrap();
    let new_id = todo_list.tasks.len() + 1;
    let mut new_task = task.into_inner();
    new_task.id = Some(new_id);
    todo_list.tasks.push(new_task.clone());
    save_tasks(&todo_list.tasks);
    Json(new_task)
}

#[post("/delete/<id>")]
fn delete_task(id: usize, todo_list: &rocket::State<Mutex<TodoList>>) -> &'static str {
    let mut todo_list = todo_list.lock().unwrap();
    todo_list.tasks.retain(|task| task.id != Some(id));
    save_tasks(&todo_list.tasks);
    "Task deleted"
}

#[post("/edit/<id>", format = "application/json", data = "<task>")]
fn edit_task(id: usize, task: Json<Task>, todo_list: &rocket::State<Mutex<TodoList>>) -> Option<Json<Task>> {
    let mut todo_list = todo_list.lock().unwrap();

    if let Some(existing_task) = todo_list.tasks.iter_mut().find(|t| t.id == Some(id) && !t.completed) {
        existing_task.title = task.title.clone();
        existing_task.description = task.description.clone();
        save_tasks(&load_tasks());
        return Some(Json(existing_task.clone()));
    }

    None
}

#[post("/complete/<id>")]
fn complete_task(id: usize, todo_list: &rocket::State<Mutex<TodoList>>) -> &'static str {
    let mut todo_list = todo_list.lock().unwrap();
    if let Some(task) = todo_list.tasks.iter_mut().find(|t| t.id == Some(id)) {
        task.completed = true;
        save_tasks(&todo_list.tasks);
    }
    "Task marked as completed"
}

#[post("/export")]
fn export_tasks(todo_list: &rocket::State<Mutex<TodoList>>) -> Option<String> {
    let tasks = todo_list.lock().unwrap().tasks.clone();
    let json = serde_json::to_string(&tasks).ok()?;
    Some(json)
}

#[post("/import", format = "application/json", data = "<tasks_json>")]
fn import_tasks(tasks_json: Json<Vec<Task>>, todo_list: &rocket::State<Mutex<TodoList>>) {
    let mut todo_list = todo_list.lock().unwrap();
    todo_list.tasks = tasks_json.into_inner();
    save_tasks(&todo_list.tasks);
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, add_task, delete_task, edit_task, complete_task, export_tasks, import_tasks])
        .attach(Template::fairing())
        .manage(Mutex::new(TodoList::default()))
}
