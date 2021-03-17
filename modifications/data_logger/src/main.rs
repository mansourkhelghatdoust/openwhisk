use actix_web::{get, post, web, App, HttpServer, Responder};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use structopt::StructOpt;
use web::Path;

mod graph;
use graph::*;

mod epoch_cache;

mod flatten;
use flatten::*;

#[derive(StructOpt, Clone)]
struct Arguments {
    /// The port to listen on
    port: u16,
    /// Application config file (JSON)
    appplication_config: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct LogEntry {
    action: String,
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
        error!("Did not find memory for action {}, returning default", action);
        web::Json(Memory { memory: 256 })
    }

}

#[post("/calls/{application_id}/{caller}/{callee}")]
async fn calls(
    Path((application_id, caller, callee)): Path<(String, String, String)>,
    graph: CallGraph,
) -> impl Responder {

    info!("Received: {} {} {}", application_id, caller, callee);
    
    let mut graph = graph.lock().unwrap();
    graph.edge(&caller, &callee).call_count += 1;

    "Ok"
}

#[post("/logs")]
async fn post_log(entry: web::Json<LogEntry>, graph: CallGraph) -> impl Responder {
    info!("Received: {:?}", *entry);

    
    let mut graph = graph.lock().unwrap();
    let node = graph.get_node(&entry.action);
    node.invoke_count += 1;
    node.buffer.add(entry.actual);

    "Ok"
}

type CallGraph = web::Data<Mutex<Graph<EdgeInfo, ActionInfo>>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Arguments::from_args();
    let port = args.port;

    let app: Application =
        serde_json::from_str(&std::fs::read_to_string(args.appplication_config)?).unwrap();

    let app = web::Data::new(app);

    let call_graph: CallGraph = web::Data::new(Mutex::new(Graph::new()));

    env_logger::Builder::from_default_env().init();
    println!("Listening on http://0.0.0.0:{}/logs", port);

    HttpServer::new(move || {
        App::new()
            .service(post_log)
            .service(calls)
            .service(get_memory)
            .app_data(app.clone())
            .app_data(call_graph.clone())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
