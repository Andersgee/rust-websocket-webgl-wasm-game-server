mod messages;
mod server;
mod session;

use actix::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use session::Session;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::Server>>,
) -> Result<HttpResponse, Error> {
    let server_addr = srv.get_ref().clone();
    ws::start(Session::new(server_addr), &req, stream)
}

async fn get_count(count: web::Data<AtomicUsize>) -> impl Responder {
    let current_count = count.load(Ordering::SeqCst);
    format!("Visitors: {current_count}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("main running");
    let app_state = Arc::new(AtomicUsize::new(0));
    let server_addr = server::Server::new(app_state.clone()).start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(server_addr.clone()))
            .route("/count", web::get().to(get_count))
            .route("/ws", web::get().to(websocket_route))
    })
    .workers(2)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
