use actix_web::{get, post, web, App, HttpServer, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use structopt::StructOpt;
use web::Path;

#[derive(StructOpt, Clone)]
struct Arguments {
    /// The file to store logs in
    log_file: String,
    /// The file to store calls in
    calls_file: String,
    /// The port to listen on
    port: u16,
    /// Application config file (JSON)
    appplication_config: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct LogEntry {
    action: String,
    estimated: u64,
    actual: u64,
}

#[derive(Serialize)]
struct Memory {
    memory: u64,
}

#[derive(Deserialize)]
struct Application {
    credentials: String,
    application_id: u64,
    actions: HashMap<String, Action>,
}

#[derive(Deserialize)]
struct Action {
    memory: u64,
    actions: Vec<Transition>,
}

#[derive(Deserialize)]
struct Transition {
    action_name: String,
    probability: f32,
}

#[get("/{action}/memory")]
async fn get_memory(Path(action): Path<String>,
    application: web::Data<Application>) -> impl Responder {
    
    info!("Got memory request for action: {}", action);
    if let Some(app_action) = application.actions.get(&action) {
        info!("Found memory for action, returning memory: {}", app_action.memory);
        web::Json(Memory { memory: app_action.memory })
    } else {
        info!("Did not find memory for action, returning default");
        web::Json(Memory { memory: 256 })
    }

}

#[post("/calls/{application_id}/{caller}/{callee}")]
async fn calls(
    log_file: web::Data<CallsFile>,
    Path((application_id, caller, callee)): Path<(String, String, String)>,
) -> impl Responder {
    let mut file = log_file.0.lock().unwrap();
    writeln!(file, "{},{},{}", application_id, caller, callee).unwrap();
    info!("Received: {} {} {}", application_id, caller, callee);
    "Ok"
}

#[post("/logs")]
async fn post_log(entry: web::Json<LogEntry>, log_file: web::Data<LogFile>) -> impl Responder {
    let mut file = log_file.lock().unwrap();
    writeln!(
        file,
        "{},{},{}",
        entry.action, entry.estimated, entry.actual
    )
    .unwrap();
    info!("Received: {:?}", *entry);
    "Ok"
}

type LogFile = Mutex<File>;
struct CallsFile(Mutex<File>);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Arguments::from_args();
    let port = args.port;

    let app: Application =
        serde_json::from_str(&std::fs::read_to_string(args.appplication_config)?).unwrap();

    let app = web::Data::new(app);

    let file = web::Data::new(Mutex::new(std::fs::File::create(args.log_file)?));
    let calls_file = web::Data::new(CallsFile(Mutex::new(std::fs::File::create(
        args.calls_file,
    )?)));
    writeln!(file.lock().unwrap(), "action,estimated,actual").unwrap();
    writeln!(calls_file.0.lock().unwrap(), "application_id,caller,callee").unwrap();

    env_logger::Builder::from_default_env().init();
    println!("Listening on http://0.0.0.0:{}/logs", port);

    HttpServer::new(move || {
        App::new()
            .service(post_log)
            .service(calls)
            .service(get_memory)
            .app_data(file.clone())
            .app_data(calls_file.clone())
            .app_data(app.clone())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
