use std::{collections::HashMap};

use actix_web::HttpResponse;
use deadpool_redis::{PoolError, Pool as RedisPool, Connection as RedisConnection};
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};

use crate::{db::Campus, util::log_error};

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisVoterType {
      pub token: String,
      pub campus: Campus
}

pub async fn get_voters_data_redis(redis_pool: &RedisPool) -> Result<HashMap<String, RedisVoterType>, HttpResponse> {
      let redis_connection_result: Result<RedisConnection, PoolError>  = redis_pool.get().await;
      let mut redis_connection: RedisConnection = match redis_connection_result {
            Ok(connection) => connection,
            Err(err) => {
                  log_error("GetTokenRedis", format!("There's an error when trying to get redis pool. Error: {}", err.to_string()).as_str());
                  return Err(HttpResponse::InternalServerError().finish());
            }
      };

      let redis_voter_tokens: Result<HashMap<String, String>, redis::RedisError>  = redis_connection.hgetall("voter_token_reset").await;
      let redis_voter_tokens: HashMap<String, String> = match redis_voter_tokens {
            Ok(data) => data,
            Err(err) => {
                  log_error("GetTokenRedis", format!("There's an error when trying to get redis voter. Error: {}", err.to_string()).as_str());
                  return Err(HttpResponse::InternalServerError().finish());
            }
      };

      let mut redis_voter_tokens_deserialized: HashMap<String, RedisVoterType> = HashMap::new();
      for redis_token in redis_voter_tokens.iter() {
            let deserialized_voter_data: Result<RedisVoterType, serde_json::Error> = serde_json::from_str::<RedisVoterType>(redis_token.1);
            let deserialized_voter_data = match deserialized_voter_data {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("GetTokenRedis", format!("There's an error when trying to get deserialize redis voter data. Error: {}", err.to_string()).as_str());
                        return Err(HttpResponse::InternalServerError().finish());
                  }
            };

            redis_voter_tokens_deserialized.insert(redis_token.0.clone(), deserialized_voter_data);
      }

      Ok(redis_voter_tokens_deserialized)
}

pub async fn set_voters_data_redis(redis_pool: &RedisPool, voter_name: &str, new_voter_token: &str, campus_name: &Campus) -> Result<(), HttpResponse> {
      let redis_connection_result: Result<deadpool_redis::Connection, PoolError>  = redis_pool.get().await;
      let mut redis_connection: deadpool_redis::Connection = match redis_connection_result {
            Ok(connection) => connection,
            Err(err) => {
                  log_error("SetTokenRedis", format!("There's an error when trying to get redis pool. Error: {}", err.to_string()).as_str());
                  return Err(HttpResponse::InternalServerError().finish());
            }
      };


      let serialized_data: Result<String, serde_json::Error> = serde_json::to_string(&RedisVoterType {
            campus: campus_name.clone(),
            token: new_voter_token.to_string()
      });
      let serialized_data: String = match serialized_data {
            Ok(data) => data,
            Err(err) => {
                  log_error("SetTokenRedis", format!("There's an error when trying to serialize redis voter data. Error: {}", err.to_string()).as_str());
                  return Err(HttpResponse::InternalServerError().finish());
            }
      };

      let insert_result: Result<(), RedisError> = redis_connection.hset("voter_token_reset", voter_name.to_string(), serialized_data).await;

      match insert_result {
            Ok(_) => (),
            Err(err) => {
                  log_error("SetTokenRedis", format!("There's an error when trying to reset a voter token to Redis. Error: {}", err.to_string()).as_str());
                  return Err(HttpResponse::InternalServerError().finish());
            }
      }


      Ok(())
}
