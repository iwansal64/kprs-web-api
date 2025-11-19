use time::{OffsetDateTime, macros::{format_description, offset}};
use rand::Rng;

static DATETIME_FMT: &[time::format_description::FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");

pub fn get_time() -> String {
      let utc = OffsetDateTime::now_utc();
      let local = utc.to_offset(offset!(+7));
      
      return local.format(DATETIME_FMT).unwrap().to_string();
}

pub fn log_something(scope_title: &str, message: &str) {
      println!("[{}] [{}] {}", get_time(), scope_title, message);
}

pub fn log_error(scope_title: &str, message: &str) {
      println!("[{}] [ERROR] [{}] {}", get_time(), scope_title, message);
}

static TOKEN_LENGTH: usize = 5;

pub fn generate_token() -> String {
      let mut result: String = String::new();
      let mut rng = rand::rng();

      // Iterate for each characters
      // for i in 65..(65+52) {
      //       if i > 25 { i += 6; }
            
      // }
      for i in 0..=(TOKEN_LENGTH) {
            let mut random_index = rng.random_range(65..(65+52));
            if random_index > 90 { random_index += 6; }

            if let Some(data) = char::from_u32(random_index) {
                  result.insert(i, data);
            }
            else {
                  result.insert(i, 'A');
            }
      }

      result
}