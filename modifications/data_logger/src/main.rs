use actix_web::{post, web, App, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use std::sync::Mutex;
use std::fs::File;
use log::info;
use std::io::Write;

#[derive(StructOpt, Clone)]
struct Arguments {
    /// The file to store logs in
    log_file: String,
    /// The port to listen on
    port: u16,
}

#[derive(Deserialize, Serialize, Debug)]
struct LogEntry {
    action: String,
    estimated: u64,
    actual: u64,
}

#[post("/logs")]
async fn post_log(entry: web::Json<LogEntry>, log_file: web::Data<LogFile>) -> impl Responder {
    let mut file = log_file.lock().unwrap();
    writeln!(file, "{},{},{}", entry.action, entry.estimated, entry.actual).unwrap();
    info!("Received: {:?}", *entry);
    "Ok"
}

type LogFile = Mutex<File>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Arguments::from_args();
    let port = args.port;
    let file = web::Data::new(Mutex::new(std::fs::File::create(args.log_file)?));
    writeln!(file.lock().unwrap(), "action,estimated,actual").unwrap();
    env_logger::Builder::from_default_env().init();
    println!("Listening on http://0.0.0.0:{}/logs", port);

    HttpServer::new(move || App::new().service(post_log).app_data(file.clone()))
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
