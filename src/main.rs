use std::process::Command;

use hooked::data::Payload;
use hooked::middleware::Auth;
use hooked::Config;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    hooked::init_logger();
    info!("loading config file");
    let config =
        Config::acquire().unwrap_or_else(|err| hooked::abort("unable to load config file!", err));
    let mut app = tide::with_state(config.clone());
    app.with(Auth::new(&config.secret));
    app.at("/")
        .post(|mut req: tide::Request<Config>| async move {
            let header = match req.header("X-GitHub-Event") {
                Some(header) => header.as_str(),
                None => {
                    warn!("missing event type!");
                    return Ok(tide::Response::new(tide::StatusCode::BadRequest));
                }
            };
            let event = format!("{}", &header);
            let payload: Payload = match req.body_json().await {
                Ok(payload) => payload,
                Err(err) => {
                    error!("malformed payload detected! Reason: {}", err);
                    return Ok(tide::Response::new(tide::StatusCode::BadRequest));
                }
            };
            info!("{} {}", payload.repository.id, event);
            for hook in &req.state().hooks {
                if hook.id.eq(&payload.repository.id) && hook.event.eq(&event) {
                    let (command, args) = hook.exec.split_once(' ').unwrap();
                    Command::new(command).args(args.split(' ')).spawn().unwrap();
                }
            }
            Ok(tide::Response::new(tide::StatusCode::Ok))
        });
    info!("starting server on {}:{}", config.host, config.port);
    app.listen(&format!("{}:{}", config.host, config.port))
        .await
        .unwrap_or_else(|err| hooked::abort("failed to start server!", err.into()));
}
