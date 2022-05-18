//! Public Types of the client module

use derive_getters::Getters;

/// The type representing a Bilibili's message received 
/// by the client.
pub enum BiliMessage {
	/// Someone sent a Bullet Screen Comment
	Danmu(DanmuMessage),
}

/// The type representing a bullet screen message
#[derive(Debug, Getters)]
pub struct DanmuMessage {
	// sender's uid
	uid: u64,
	// sender's user name
	uname: String,
	// content of the bullet screen comment
	content: String,
	// is this message auto-generated for
	// gift sending?
	// (自动生成的xxx投喂了xxx的弹幕)
	is_gift_auto: bool,
	// timestamp the message is sent
	sent_time: u64,
	// is sender a manager(房管) of the room?
	is_manager: bool,
	// is sender a vip(老爷)?
	is_vip: bool,
	// is sender a svip(年费老爷)?
	is_svip: bool,
	// 正式会员
	is_full_member: bool,

	// 勋章，可能未佩戴
	#[getter(skip)]
	medal: Option<Medal>,

	// 用户等级 & 等级排名
	ul: u64,
	ul_rank: String,

	// 舰队身份
	guard: GuardType,
}

impl DanmuMessage{

	pub fn has_medal(&self) -> bool {
		self.medal.is_some()
	}

	pub fn medal_level(&self) -> Option<u64> {
		self.medal.as_ref().map(|m| m.level)
	}

	pub fn medal_name(&self) -> Option<String> {
		self.medal.as_ref().map(|m| m.name.clone())
	}

	pub fn medal_streamer_name(&self) -> Option<String> {
		self.medal.as_ref().map(|m| m.streamer_name.clone())
	}

	pub fn medal_streamer_roomid(&self) -> Option<u64> {
		self.medal.as_ref().map(|m| m.streamer_roomid)
	}
}

#[derive(Debug)]
struct Medal {
	level: u64,
	name: String,
	streamer_name: String,
	streamer_roomid: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardType {
	// 舰长
	Captain,
	// 提督
	Admiral,
	// 总督
	Governor
}

