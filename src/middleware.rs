use actix_web::{
      body::MessageBody,
      dev::{ServiceRequest, ServiceResponse},
      middleware::Next,
      Error,
};

use crate::util::log_something;


pub async fn middleware(req: ServiceRequest, next: Next<impl MessageBody>) -> Result<ServiceResponse<impl MessageBody>, Error> {
      log_something("Middleware", format!("Got a Request! Destination: {}", req.uri().to_string()).as_str());
      next.call(req).await
}


