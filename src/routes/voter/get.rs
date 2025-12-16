use crate::{data::voter::get_voters_data, util::log_error};
use actix_web::{HttpResponse, cookie::Cookie, post, web};
use deadpool_redis::{Pool as RedisPool, PoolError};
use redis::{AsyncCommands, RedisError};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserData {
    fullname: String,
    token: String,
}

#[post("/voter/get")]
pub async fn post(redis_pool: web::Data<RedisPool>, data: web::Json<UserData>) -> HttpResponse {
    // Get the targetted user data token
    let data = data.into_inner();
    let target_user_fullname = data.fullname;
    let target_user_token = data.token;

    // Check in the users hashmap
    let static_voters_data = get_voters_data();
    let locked_static_voters_data = static_voters_data.read().await;
    let static_voter_data = locked_static_voters_data.get(&target_user_fullname);
    let static_voter_data = match static_voter_data {
        Some(data) => data,
        None => {
            return HttpResponse::NotFound().finish();
        }
    };

    // Check in the Redis if the token is resetted
    let redis_connection_result: Result<deadpool_redis::Connection, PoolError> =
        redis_pool.get().await;
    let mut redis_connection: deadpool_redis::Connection = match redis_connection_result {
        Ok(connection) => connection,
        Err(err) => {
            log_error(
                "VoterGet",
                format!(
                    "There's an error when trying to get admin redis pool. Error: {}",
                    err.to_string()
                )
                .as_str(),
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    let redis_user_token_result: Result<Option<String>, RedisError> = redis_connection
        .hget("voter_token_reset", target_user_fullname)
        .await;
    let redis_user_token_maybe: Option<String> = match redis_user_token_result {
        Ok(data) => data,
        Err(err) => {
            log_error("UserGet", err.to_string().as_str());
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Check for token in Redis
    if let Some(redis_user_token) = &redis_user_token_maybe
        && &target_user_token != redis_user_token
    {
        return HttpResponse::Unauthorized().finish();
    }

    // If there's no targetted user token in redis check with the default data user token
    if redis_user_token_maybe.is_none() && target_user_token != static_voter_data.token {
        return HttpResponse::Unauthorized().finish();
    }

    // Create response object and add cookie
    let cookie_user_token = Cookie::build("voter_token", target_user_token)
        .path("/")
        .secure(true)
        .http_only(true)
        .finish();

    // Return the response
    HttpResponse::Ok().cookie(cookie_user_token).json(static_voter_data)
}
