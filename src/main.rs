use std::process::Command;

use hmac::Mac;
use tide_rustls::TlsListener;
use webhook::data::Payload;
use webhook::Config;

#[macro_use]
extern crate log;

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

pub struct AuthMiddleware {
    secret: String,
}

impl AuthMiddleware {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for AuthMiddleware {
    async fn handle(
        &self,
        mut req: tide::Request<State>,
        next: tide::Next<'_, State>,
    ) -> tide::Result {
        let header = match req.header("X-Hub-Signature-256") {
            Some(header) => header.as_str(),
            None => {
                warn!("unauthorized request detected!");
                return Ok(tide::Response::new(tide::StatusCode::Unauthorized));
            }
        };
        let secret_pub = format!("{}", &header);
        let payload = req.body_string().await?;
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())?;
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let secret_prv = format!("sha256={}", hex::encode(&result.into_bytes()));
        if !secret_prv.eq(&secret_pub) {
            warn!("unauthorized request detected!");
            return Ok(tide::Response::new(tide::StatusCode::Unauthorized));
        }
        req.set_body(payload);
        let res = next.run(req).await;
        Ok(res)
    }
}

#[tokio::main]
async fn main() {
    webhook::init_logger();
    info!("loading config file");
    let config =
        Config::acquire().unwrap_or_else(|err| webhook::abort("unable to load config file!", err));
    let mut app = tide::with_state(config.clone());
    let auth_middleware = AuthMiddleware::new(&config.secret);
    app.with(auth_middleware);
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
                    info!("start");
                    Command::new(hook.exec.clone()).spawn().unwrap();
                }
            }
            Ok(tide::Response::new(tide::StatusCode::Ok))
        });
    info!("starting server on {}:{}", config.host, config.port);
    app.listen(
        TlsListener::build()
            .addrs(format!("{}:{}", config.host, config.port))
            .cert(config.cert_path)
            .key(config.key_path),
    )
    .await
    .unwrap_or_else(|err| webhook::abort("failed to start server!", err.into()));
}
