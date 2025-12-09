use sqlx::{FromRow, Pool, Postgres, postgres::PgQueryResult};

#[derive(FromRow)]
pub struct Voter {
    pub token: String,
    pub name: String,
}

pub async fn get_all_users(pool: &Pool<Postgres>) -> sqlx::Result<Vec<Voter>> {
      sqlx::query_as::<_, Voter>(
          "SELECT * FROM voters"
      )
      .fetch_all(pool)
      .await
}

pub async fn get_user_by_token(pool: &Pool<Postgres>, token: String) -> sqlx::Result<Voter> {
      sqlx::query_as::<_, Voter>(
            "SELECT * FROM voters WHERE token = $1"
      )
      .bind(token)
      .fetch_one(pool)
      .await
}



#[derive(FromRow)]
pub struct Candidate {
      pub name: String
}

pub async fn get_all_candidates(pool: &Pool<Postgres>) -> sqlx::Result<Vec<Candidate>> {
      sqlx::query_as::<_, Candidate>(
          "SELECT * FROM candidates"
      )
      .fetch_all(pool)
      .await
}



#[derive(FromRow)]
pub struct Vote {
      pub voter_name: String,
      pub candidate_name: String
}

pub async fn get_all_votes(pool: &Pool<Postgres>) -> sqlx::Result<Vec<Vote>> {
      sqlx::query_as::<_, Vote>(
          "SELECT * FROM votes"
      )
      .fetch_all(pool)
      .await
}

pub async fn insert_vote(pool: &Pool<Postgres>, voter_name: String, candidate_name: String) -> Result<PgQueryResult, sqlx::Error> {
      sqlx::query("INSERT INTO votes(voter_name, candidate_name) VALUES ($1, $2);")
            .bind(voter_name)
            .bind(candidate_name)
            .execute(pool)
            .await
}

pub async fn remove_vote(pool: &Pool<Postgres>, voter_name: &str) -> Result<PgQueryResult, sqlx::Error> {
      sqlx::query("DELETE FROM votes WHERE voter_name = $1")
            .bind(voter_name)
            .execute(pool)
            .await
}
