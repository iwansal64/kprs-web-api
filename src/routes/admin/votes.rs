use std::{collections::HashMap, env, sync::atomic::{AtomicUsize, Ordering}};

use actix_web::{HttpRequest, HttpResponse, get};
use serde::Serialize;

use crate::{data::vote::get_votes_count, util::log_error};

#[derive(Serialize)]
struct GetBodyRequestType {
      votes_data: HashMap<String, usize>
}

#[get("/admin/votes")]
pub async fn get(req: HttpRequest) -> HttpResponse {
      // Verify the admin token from cookies
      let admin_token_cookie = req.cookie("admin_token");
      let admin_token_cookie = match admin_token_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => {
                  return HttpResponse::NotFound().finish();
            }
      };

      let valid_admin_token = env::var("ADMIN_TOKEN");
      let valid_admin_token = match valid_admin_token {
            Ok(data) => data,
            Err(err) => {
                  log_error("PostReset", format!("There's an error when trying to get admin token from ENV. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      };


      if admin_token_cookie != valid_admin_token {
            return HttpResponse::Unauthorized().finish();
      }


      // Get the static votes data
      let static_votes_data: &HashMap<String, AtomicUsize> = get_votes_count().await;


      HttpResponse::Ok()
            .json(GetBodyRequestType {
                  votes_data: static_votes_data.iter().map(|data| (data.0.clone(), data.1.load(Ordering::Relaxed))).collect::<HashMap<String, usize>>()
            })
}
