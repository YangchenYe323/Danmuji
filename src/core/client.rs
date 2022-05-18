use crate::config::WsConfig;
use byteorder::{BigEndian, WriteBytesExt};
use rocket::serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Arc, Mutex,
    },
    thread::JoinHandle,
};
use websocket::{ClientBuilder, Message};

pub struct WebsocketHandle {
    worker: JoinHandle<()>,
    shutdown: Arc<AtomicBool>,

    // downstream consumer of the websocket message
    downstream: Arc<Mutex<Option<Receiver<String>>>>,
}

/// The first packet body sent to Bilibili Live
/// when websocket connection is established
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct FirstSecurityData {
    clientver: String,
    platform: String,
    protover: u64,
    uid: u64,
    roomid: i64,
    #[serde(rename = "type")]
    type_: u64,
}

impl WebsocketHandle {
    pub fn new(config: &WsConfig, room_id: i64) -> WebsocketHandle {
        let shutdown = Arc::new(AtomicBool::new(false));

        let st = Arc::clone(&shutdown);
        let url = config.get_ws_url();
        let url = url.unwrap();

        let handle = std::thread::spawn(move || {
            println!("{:?}", url);
            let mut client = ClientBuilder::new(&url)
                .unwrap()
                .connect_insecure()
                .unwrap();

            println!("Connection Established");

            // send enter message
            let security_data = FirstSecurityData {
                clientver: "1.14.0".to_string(),
                platform: "web".to_string(),
                protover: 1,
                uid: 0,
                roomid: room_id,
                type_: 2,
            };
            let mut data_bytes = serde_json::to_vec(&security_data).unwrap();
            let data_length = data_bytes.len();
            // header format:
            // offset    length    type    endian    name           note
            // 0         4         i32     Big       packet-length
            // 4         2         i16     Big       header-length  Fixed 16
            // 6         2         i16     Big       proto-version
            // 8         4         i32     Big       Opertion Type
            // 12        4         i32     Big       Seq ID         Fixed 1
            // reference: https://github.com/lovelyyoshino/Bilibili-Live-API/blob/master/API.WebSocket.md
            let mut header = vec![];
            // write packet length = header length + data length
            header
                .write_u32::<BigEndian>(16 + data_length as u32)
                .unwrap();
            // write header length
            header.write_u16::<BigEndian>(16).unwrap();
            // write proto-version 1
            header.write_u16::<BigEndian>(1).unwrap();
            // write operatin type: enter room = 7
            header.write_u32::<BigEndian>(7).unwrap();
            // write seq id
            header.write_u32::<BigEndian>(1).unwrap();
            println!("header: {:?}", header);

            header.append(&mut data_bytes);

            client.send_message(&Message::binary(header)).unwrap();

            for message in client.incoming_messages() {
                // terminate
                if st.load(Ordering::Relaxed) {
                    break;
                }

                // parse message and send to downstream

                println!("Recv: {:?}", message.unwrap());
            }
        });

        WebsocketHandle {
            worker: handle,
            shutdown,
            downstream: Arc::new(Mutex::new(None)),
        }
    }

    pub fn shutdown(self) -> Option<Receiver<String>> {
        self.shutdown.store(true, Ordering::Relaxed);
        // return downstream so that it can be plugged into
        // other clients
        self.downstream.lock().unwrap().take()
    }

    pub fn set_downstream(&self, downstream: Receiver<String>) {
        let mut ds = self.downstream.lock().unwrap();
        *ds = Some(downstream);
    }
}
