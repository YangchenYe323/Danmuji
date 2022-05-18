use super::Command;
use crate::{BulletScreenConfig, Result, RoomConfig, UserConfig};
use std::{
    marker::PhantomData,
    sync::{mpsc, Arc, Mutex},
};

pub trait DanmujiState {}

pub struct Init;
impl DanmujiState for Init {}
pub struct LoggedIn;
impl DanmujiState for LoggedIn {}
pub struct Connected;
impl DanmujiState for Connected {}

pub struct DanmujiCore<S: DanmujiState> {
    rx: mpsc::Receiver<Command>,
    user: Option<UserConfig>,
    room: Option<RoomConfig>,
    bc: Option<BulletScreenConfig>,

    _phantom: PhantomData<S>,
}

impl DanmujiCore<Init> {
    pub fn new(rx: mpsc::Receiver<Command>) -> Self {
        Self {
            rx,
            user: None,
            room: None,
            bc: None,
            _phantom: PhantomData,
        }
    }

    pub async fn login(self, raw_cookie: String) -> Result<DanmujiCore<LoggedIn>> {
        let user = UserConfig::fetch(raw_cookie).await?;
        Ok(DanmujiCore::<LoggedIn> {
            rx: self.rx,
            user: Some(user),
            room: None,
            bc: None,
            _phantom: PhantomData,
        })
    }
}

impl DanmujiCore<LoggedIn> {
    pub async fn connect(self, room_id: u64) -> Result<DanmujiCore<Connected>> {
        let room =
            RoomConfig::fetch(room_id, self.user.as_ref().unwrap().raw_cookie.as_str()).await?;

        let bc = BulletScreenConfig::fetch(&room, &self.user.as_ref().unwrap()).await?;

        Ok(DanmujiCore::<Connected> {
            rx: self.rx,
            user: self.user,
            room: Some(room),
            bc: Some(bc),
            _phantom: PhantomData,
        })
    }
}
