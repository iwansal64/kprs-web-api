mod token;
mod reset;
mod votes;
mod login;
mod check;
mod simple_votes;

pub use self::token::get as admin_token_api;
pub use self::reset::post as admin_reset_api;
pub use self::votes::get as admin_votes_api;
pub use self::login::post as admin_login_api;
pub use self::check::post as admin_check_api;
pub use self::simple_votes::post as admin_votes_simple_api;
