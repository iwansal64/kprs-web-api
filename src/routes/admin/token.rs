use std::{collections::HashMap, env};
use actix_web::{HttpRequest, HttpResponse, get, web::{self, Json}};
use dashmap::DashMap;
use deadpool_redis::{Pool as RedisPool, Connection as RedisConnection};
use redis::AsyncCommands;
use serde::Serialize;

use crate::{data::voter::get_voters_data, util::log_error};

#[derive(Serialize)]
struct GetTokenResponseType {
      changed_voter_tokens: HashMap<String, String>,
      static_voter_tokens: HashMap<String, String>
}

#[get("/admin/token")]
pub async fn get(req: HttpRequest, redis_pool: web::Data<RedisPool>) -> HttpResponse {
      // Verify the admin token from cookie
      let admin_token_cookie = req.cookie("admin_token");
      let admin_token_cookie = match admin_token_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => {
                  return HttpResponse::NotFound().finish();
            }
      };
      
      let valid_admin_token = env::var("ADMIN_TOKEN").unwrap();
      if admin_token_cookie != valid_admin_token {
            return HttpResponse::Unauthorized().finish();
      }


      // Get the token data from Redis
      let mut redis_connection: RedisConnection = redis_pool.get().await.unwrap();
      let redis_voter_tokens: Result<HashMap<String, String>, redis::RedisError>  = redis_connection.hgetall("voter_token_reset").await;
      let redis_voter_tokens: HashMap<String, String> = match redis_voter_tokens {
            Ok(data) => data,
            Err(err) => {
                  log_error("GetToken", format!("There's an error when trying to get redis voter. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      };



      // Get the token data from static
      let static_voter_tokens: &DashMap<String, String> = get_voters_data().await;
      let static_voter_tokens: HashMap<String, String> = static_voter_tokens.iter().map(|data| (data.key().clone(), data.value().clone())).collect();


      // Return the token data
      let mut response = HttpResponse::Ok();

      response.json(Json(GetTokenResponseType {
            changed_voter_tokens: redis_voter_tokens,
            static_voter_tokens: static_voter_tokens
      }))
}