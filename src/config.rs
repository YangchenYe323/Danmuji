use rocket::serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserConfig {
	// cookie used to access bilibili server
	pub cookie: String,
	// user information
	pub user: Option<User>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
	pub uid: u64,
	pub uname: String,
	pub silver: u64,
	pub gold: u64,
	pub face: String,
	pub achieve: u64,
	pub vip: u64,
	pub svip: u64,
	pub user_level: u64,
	pub user_next_level: u64,
	pub user_intimacy: u64,
	pub user_next_intimacy: u64,
	pub user_level_rank: u64,
	pub user_charged: u64,
	pub billCoin: u64,
}