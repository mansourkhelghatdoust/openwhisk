use actix_web::{post, web, App, HttpServer, Responder};
use log::info;
use serde::{Deserialize, Serialize};
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
}

#[derive(Deserialize, Serialize, Debug)]
struct LogEntry {
    action: String,
    estimated: u64,
    actual: u64,
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
    let file = web::Data::new(Mutex::new(std::fs::File::create(args.log_file)?));
    let calls_file = web::Data::new(CallsFile(Mutex::new(std::fs::File::create(args.calls_file)?)));
    writeln!(file.lock().unwrap(), "action,estimated,actual").unwrap();
    writeln!(calls_file.0.lock().unwrap(), "application_id,caller,callee").unwrap();

    env_logger::Builder::from_default_env().init();
    println!("Listening on http://0.0.0.0:{}/logs", port);

    HttpServer::new(move || {
        App::new()
            .service(post_log)
            .service(calls)
            .app_data(file.clone())
            .app_data(calls_file.clone())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
