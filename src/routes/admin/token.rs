use std::{collections::HashMap, env, sync::Arc};
use actix_web::{HttpRequest, HttpResponse, get, web::{self, Json}};
use deadpool_redis::{Connection as RedisConnection, Pool as RedisPool, PoolError};
use redis::AsyncCommands;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::{data::voter::get_voters_data, db::Voter, util::log_error};

#[derive(Serialize)]
struct GetTokenResponseType {
      changed_voter_tokens: HashMap<String, String>,
      static_voter_data: HashMap<String, Voter>
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


      // Get the token data from Redis
      let redis_connection_result: Result<RedisConnection, PoolError>  = redis_pool.get().await;
      let mut redis_connection: RedisConnection = match redis_connection_result {
            Ok(connection) => connection,
            Err(err) => {
                  log_error("PostReset", format!("There's an error when trying to get admin redis pool. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      };

      let redis_voter_tokens: Result<HashMap<String, String>, redis::RedisError>  = redis_connection.hgetall("voter_token_reset").await;
      let redis_voter_tokens: HashMap<String, String> = match redis_voter_tokens {
            Ok(data) => data,
            Err(err) => {
                  log_error("GetToken", format!("There's an error when trying to get redis voter. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      };



      // Get the token data from static
      let static_voter_tokens: Arc<RwLock<HashMap<String, Voter>>> = get_voters_data();
      let locked_static_voter_tokens = static_voter_tokens.read().await.clone();


      // Return the token data
      let mut response = HttpResponse::Ok();

      response.json(Json(GetTokenResponseType {
            changed_voter_tokens: redis_voter_tokens,
            static_voter_data: locked_static_voter_tokens
      }))
}
