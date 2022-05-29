use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::client::BiliMessage;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use tracing::error;

use super::DanmujiPlugin;

/// The Plugin Executor component of Danmuji serves as the bridge
/// between [BiliClient], which collects messages from Bilibili's Websocket,
/// and [DanmujiSender], which sends our generated danmu messages. The executor
/// is responsible for executing all the plugins to generate our message(e.g., gift thanks)
/// and feeds these messages to the sender
pub struct DanmujiPluginExecutor {
    shutdown: Arc<AtomicBool>,
    plugins: Arc<Mutex<HashMap<&'static str, Box<dyn DanmujiPlugin>>>>,
}

impl DanmujiPluginExecutor {
    pub fn new(upstream: Receiver<BiliMessage>, downtream: UnboundedSender<String>) -> Self {
        let executor = Self {
            shutdown: Arc::new(AtomicBool::new(false)),
            plugins: Arc::new(Mutex::new(HashMap::new())),
        };

        tokio::spawn(start_executor(
            executor.shutdown.clone(),
            upstream,
            downtream,
            executor.plugins.clone(),
        ));

        executor
    }

    pub async fn update_plugin(&self, name: &'static str, plugin: impl DanmujiPlugin) {
        let mut plugins = self.plugins.lock().await;
        plugins.insert(name, Box::new(plugin));
    }
}

impl Drop for DanmujiPluginExecutor {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

async fn start_executor(
    shutdown: Arc<AtomicBool>,
    mut upstream: Receiver<BiliMessage>,
    downstream: UnboundedSender<String>,
    plugins: Arc<Mutex<HashMap<&'static str, Box<dyn DanmujiPlugin>>>>,
) {
    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        let msg = upstream.recv().await;
        if let Err(err) = msg {
            error!("Sending End dropped: {}", err);
            break;
        }

        let msg = msg.unwrap();
        {
            let mut plugins = plugins.lock().await;
            for plugin in plugins.values_mut() {
                let reply = plugin.process_mesage(&msg);
                if let Some(reply) = reply {
                    if let Err(err) = downstream.send(reply) {
                        error!("Danmu Sender's Receiving End is dropped: {}", err);
                        break;
                    }
                }
            }
        }
    }
}
