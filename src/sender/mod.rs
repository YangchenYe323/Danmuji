//! This module implements the Bullet Screen Sender Component
//! of Danmuji. It receives message to send from a mpsc channel
//! and handles posting the message to the live room
//!

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::{mpsc, Mutex};
use tracing::{error, warn};

use crate::{
    config::{BulletScreenConfig, RoomConfig, UserConfig},
    DanmujiResult, USER_AGENT,
};

pub type Producer = mpsc::UnboundedReceiver<String>;

#[derive(Debug)]
pub struct DanmujiSender {
    // shutdown control flag
    shutdown: Arc<AtomicBool>,
    // sender's authentication info
    user: Arc<Mutex<Option<UserConfig>>>,
    // sender's bullet screen config info
    danmu: Arc<Mutex<Option<BulletScreenConfig>>>,
    // room config
    room: Arc<Mutex<Option<RoomConfig>>>,
}

impl DanmujiSender {
    pub fn start(upstream: Producer) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let user = Arc::new(Mutex::new(None));
        let danmu = Arc::new(Mutex::new(None));
        let room = Arc::new(Mutex::new(None));

        tokio::spawn(start_worker(
            upstream,
            shutdown.clone(),
            user.clone(),
            danmu.clone(),
            room.clone(),
        ));

        Self {
            shutdown,
            user,
            danmu,
            room,
        }
    }

    pub async fn login_user(&self, new_user: UserConfig) -> DanmujiResult<()> {
        let mut user = self.user.lock().await;
        let room = self.room.lock().await;
        let mut danmu = self.danmu.lock().await;
        // currently we only do login when user is None
        assert!(user.is_none());
        assert!(danmu.is_none());

        // fetch danmu config if room is not None
        if let Some(room) = room.as_ref() {
            let danmu_config = BulletScreenConfig::fetch(room, &new_user).await?;
            *danmu = Some(danmu_config);
        }
        *user = Some(new_user);
        Ok(())
    }

    pub async fn unlog_user(&self) {
        let mut user = self.user.lock().await;
        let mut danmu = self.danmu.lock().await;

        assert!(user.is_some());

        user.take();
        danmu.take();
    }

    pub async fn connect_room(&self, new_room: RoomConfig) -> DanmujiResult<()> {
        let user = self.user.lock().await;
        let mut room = self.room.lock().await;
        let mut danmu = self.danmu.lock().await;

        assert!(room.is_none());
        assert!(danmu.is_none());

        // fetch danmu config is user is not None
        if let Some(user) = user.as_ref() {
            let danmu_config = BulletScreenConfig::fetch(&new_room, user).await?;
            *danmu = Some(danmu_config);
        }

        *room = Some(new_room);

        Ok(())
    }

    pub async fn disconnect_room(&self) {
        let mut room = self.room.lock().await;
        let mut danmu = self.danmu.lock().await;

        assert!(room.is_some());

        room.take();
        danmu.take();
    }
}

impl Drop for DanmujiSender {
    fn drop(&mut self) {
        // signal shutdown for background thread
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

async fn start_worker(
    mut upstream: Producer,
    shutdown: Arc<AtomicBool>,
    user: Arc<Mutex<Option<UserConfig>>>,
    danmu: Arc<Mutex<Option<BulletScreenConfig>>>,
    room: Arc<Mutex<Option<RoomConfig>>>,
) {
    loop {
        // check shutdown
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        let msg = upstream.recv().await;
        if msg.is_none() {
            warn!("Sending Half has been dropped, Sender returns");
            break;
        }

        let msg = msg.unwrap();

        {
            let user = user.lock().await;
            let room = room.lock().await;
            let danmu = danmu.lock().await;

            if user.is_none() || room.is_none() || danmu.is_none() {
                continue;
            }

            let (user, room, danmu) = (
                user.as_ref().unwrap(),
                room.as_ref().unwrap(),
                danmu.as_ref().unwrap(),
            );

            let form = build_form(msg, room, user, danmu);

            // send
            let cli = reqwest::Client::new();
            let res = cli
                .post("https://api.live.bilibili.com/msg/send")
                .header("user-agent", USER_AGENT)
                .header("cookie", user.raw_cookie.as_str())
                .form(&form)
                .send()
                .await;

            match res {
                Ok(res) => {
                    println!("{:?}", res.text().await);
                }
                Err(err) => {
                    error!("Bullet Screen Post Error: {}", err)
                }
            }
        }
    }
}

fn build_form<'a>(
    msg: String,
    room: &'a RoomConfig,
    user: &'a UserConfig,
    danmu: &'a BulletScreenConfig,
) -> HashMap<&'static str, String> {
    let mut form = HashMap::new();
    form.insert("color", danmu.danmu.color.to_string());
    form.insert("fontsize", "25".to_string());
    form.insert("mode", danmu.danmu.mode.to_string());
    form.insert("msg", msg);
    let mut rnd = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    rnd.truncate(10);
    form.insert("rnd", rnd);
    form.insert("roomid", room.room_init.room_id.to_string());
    form.insert("bubble", danmu.bubble.to_string());
    form.insert("csrf_token", user.cookie.bili_jct.clone());
    form.insert("csrf", user.cookie.bili_jct.clone());
    form
}
