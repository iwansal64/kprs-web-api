use tokio::sync::OnceCell;

use crate::{db::{Candidate, get_all_candidates}, util::{log_error, log_something}};


pub static CANDIDATES_DATA: OnceCell<Vec<Candidate>> = OnceCell::const_new();

pub async fn get_candidates_data<'a>() -> &'a Vec<Candidate> {
      let data = CANDIDATES_DATA.get_or_init(async || {
            // Get the candidate data
            let db_all_candidates = get_all_candidates().await;
            let db_all_candidates: Vec<Candidate> = match db_all_candidates {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("StaticData", format!("There's an error when trying to get static data from database. Error: {}", err.to_string()).as_str());
                        return Vec::new();
                  }
            };

            // Log the success message
            log_something("StaticData", "Static candidates data successfully initialized.");

            // Return the result
            db_all_candidates
      }).await;

      return &data;
}

pub async fn init_candidates_data() {
      get_candidates_data().await;
}
