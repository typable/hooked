use std::process::Command;

use hooked::data::Payload;
use hooked::middleware::Auth;
use hooked::Config;
use tide::Request;
use tide::Response;
use tide::StatusCode;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    hooked::init_logger();
    info!("loading config file");
    let config =
        Config::acquire().unwrap_or_else(|err| hooked::abort("unable to load config file!", err));
    println!("secret: {}", config.secret);
    let mut app = tide::with_state(config.clone());
    app.with(Auth::new(&config.secret));
    app.at("/").post(|mut req: Request<Config>| async move {
        let header = match req.header("X-GitHub-Event") {
            Some(header) => header.as_str(),
            None => {
                warn!("missing event type!");
                let mut resp = Response::new(StatusCode::BadRequest);
                resp.set_body("missing event type!");
                return Ok(resp);
            }
        };
        let event = header.to_string();
        info!("event: {}", event);
        let payload: Payload = match req.body_json().await {
            Ok(payload) => payload,
            Err(err) => {
                error!("malformed payload detected! Reason: {}", err);
                let mut resp = Response::new(StatusCode::BadRequest);
                resp.set_body(format!("malformed payload detected! Reason: {}", err));
                return Ok(resp);
            }
        };
        info!("repository: {}", payload.repository.id);
        for hook in &req.state().hooks {
            if hook.id.eq(&payload.repository.id) && hook.event.eq(&event) {
                let mut parts = hook.exec.splitn(2, ' ');
                let command = parts.next().unwrap();
                let mut builder = Command::new(command);
                if let Some(args) = parts.next() {
                    builder.args(args.split(' '));
                }
                builder.spawn().unwrap();
            }
        }
        Ok(Response::new(StatusCode::Ok))
    });
    info!("starting server on {}:{}", config.host, config.port);
    app.listen(&format!("{}:{}", config.host, config.port))
        .await
        .unwrap_or_else(|err| hooked::abort("failed to start server!", err.into()));
}
