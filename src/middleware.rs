use hmac::Mac;

use crate::HmacSha256;

pub struct Auth {
    secret: String,
}

impl Auth {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for Auth {
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
