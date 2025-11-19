use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use dashmap::DashMap;
use deadpool_redis::{Pool as RedisPool, self};
use redis::{AsyncCommands, RedisError};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{data::{candidate::get_candidates_data, voter::get_voters_data, vote::get_votes_count}, db::insert_vote, util::{log_error, log_something}};

#[derive(Deserialize)]
struct VoteBodyRequest {
      candidate_fullname: String
}


#[post("/voter/vote")]
pub async fn post(body: web::Json<VoteBodyRequest>, req: HttpRequest, redis_pool: web::Data<RedisPool>, postgres_pool: web::Data<Pool<Postgres>>) -> impl Responder {
      // Get the user token from request cookies
      let cookie_user_token = req.cookie("voter_token");
      let cookie_user_token = match cookie_user_token {
            Some(data) => data.value().to_string(),
            None => {
                  return HttpResponse::Unauthorized();
            }
      };

      // Verify the token from checking into the redis database
      let mut redis_connection: deadpool_redis::Connection = redis_pool.get().await.unwrap();
      let redis_user_token_result: Result<HashMap<String, String>, RedisError>  = redis_connection.hgetall("voter_token_reset").await;
      let redis_user_tokens: HashMap<String, String> = match redis_user_token_result {
            Ok(data) => data,
            Err(err) => {
                  log_error("PostVote", err.to_string().as_str());
                  return HttpResponse::InternalServerError();
            }
      };
      let redis_user_name: Option<String> = redis_user_tokens.iter().find(|(_, v)| v == &&cookie_user_token).map(|user_data| user_data.0.clone());


      // Verify the token from checking into the redis database
      let users_data: &DashMap<String, String> =  get_voters_data().await;
      let data_user_fullname  = users_data.iter().find(|data| data.value() == &cookie_user_token); 
      let data_user_fullname: Option<String> = match data_user_fullname {
            Some(data) => Some(data.key().clone()),
            None => None
      };


      // Verify the token using this step:
      // 1. Positive if the token is inside Redis
      // 2. Negative if the token is inside of the voter inside Hashmap and not inside Redis
      // 3. Negative if the token is inside the Redis
      let redis_user_token_by_data_user_name = match &data_user_fullname {
            Some(fullname) => redis_user_tokens.get(fullname),
            None => None
      };

      if redis_user_name.is_none() && (redis_user_token_by_data_user_name.is_some() || data_user_fullname.is_none()) {
            return HttpResponse::Unauthorized();
      }

      let target_voter_fullname = redis_user_name.unwrap_or(
            data_user_fullname.unwrap()
      );


      // Get the candidate info from request data
      let request_body = body.into_inner();
      let target_candidate_fullname: String = request_body.candidate_fullname;
      

      // Verify candidate name
      let candidates_data = get_candidates_data().await;
      if !candidates_data.contains(&target_candidate_fullname) {
            log_something("PostVote", format!("{} has votes {} that is currently not registered", target_voter_fullname, target_candidate_fullname).as_str());
            return HttpResponse::BadRequest();
      }
      
      
      // Create vote record into the PostgreSQL
      let vote_record = insert_vote(&postgres_pool, target_voter_fullname.clone(), target_candidate_fullname.clone()).await;

      match vote_record {
            Ok(_) => {
                  log_something("PostVote", format!("{} has successfully votes {}", target_voter_fullname, target_candidate_fullname).as_str());
            },
            Err(err) => {
                  log_error("PostVote", format!("There's an error when trying to update vote record into the database. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError();
            }
      }


      // Get the static vote
      let static_votes_data = get_votes_count().await;
      
      // Check for invalid candidate's name
      if !static_votes_data.contains_key(&target_candidate_fullname) {
            log_error("PostVote", "The candidate is not found");
            return HttpResponse::BadRequest();
      }

      // Increment the vote data that is from hashmap
      static_votes_data.entry(target_candidate_fullname)
                        .and_modify(|data| {
                              data.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        });
      
      
      // Return OK
      HttpResponse::Ok()
}