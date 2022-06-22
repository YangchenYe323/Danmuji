//! This module contains types and functions for dealing with
//! Bilibili's Websocket Message Format.
//! For further detail please see reference:
//! https://github.com/lovelyyoshino/Bilibili-Live-API/blob/master/API.WebSocket.md
#![allow(dead_code)]

use std::io::{Cursor, Read};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::DanmujiResult;

/// Struct representing BiliBili's top-level websocket message frame.
/// An entire frame is parsed into a [BiliWebsocketMessage], which can be
/// then parsed into one or more [BiliWebsocketInner] that contains
/// better formatted structural data
///
/// Message format:
/// [16 bytes header][data]
#[derive(Debug, PartialEq, Eq)]
pub struct BiliWebsocketMessage {
    // header for the entire frame
    header: BiliWebsocketHeader,
    // raw data of this frame
    data: Vec<u8>,
}

impl BiliWebsocketMessage {
    // construct from raw data
    fn new(data: Vec<u8>, op: OpType, protocol_version: u16) -> Self {
        let packet_length = HEADER_LENGTH as u32 + data.len() as u32;
        Self {
            header: BiliWebsocketHeader::new(packet_length, protocol_version, op),
            data,
        }
    }

    // construct entry security message
    pub fn entry(room_id: i64, uid: Option<u64>) -> Self {
        let data = FirstSecurityData::new(room_id, uid);
        let data_bytes = serde_json::to_vec(&data).unwrap();

        Self::new(data_bytes, OpType::Entry, 2)
    }

    // construct heartbeat message
    pub fn heartbeat() -> Self {
        // heartbeat message has not data
        let data = vec![];

        Self::new(data, OpType::HeartBeat, 2)
    }

    // construct from binary (received from websocket server)
    pub fn from_binary(mut buf: Vec<u8>) -> DanmujiResult<Self> {
        // parse header
        let (header, _) = buf.split_at(HEADER_LENGTH as usize);
        let header = BiliWebsocketHeader::from_vec(header);

        // the rest is data
        buf.drain(0..HEADER_LENGTH as usize);

        Ok(Self { header, data: buf })
    }

    // serialize to a binary vector
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend(self.header.to_vec().iter());
        buf.extend(self.data.iter());
        buf
    }

    /// Consume the message and unpack all the inner messages
    // todo: This method should ideally return a vector of Result<BiliWebsocketInner>
    // todo: to propagate parsing error upward, so that we can get rid of the unwraps
    pub fn parse(self) -> Vec<BiliWebsocketInner> {
        let BiliWebsocketMessage { header, data } = self;
        match header.op {
            OpType::Notification => {
                match header.protocol_version {
                    // data is zlib compressed
                    2 => {
                        let mut z = ZlibDecoder::new(&data[..]);
                        let mut decompressed_buf = vec![];
                        z.read_to_end(&mut decompressed_buf).unwrap();
                        process_zlib_data(decompressed_buf)
                    }
                    // data is not compressed
                    _ => {
                        vec![BiliWebsocketInner {
                            header,
                            body: BiliWebsocketMessageBody::Notification(
                                serde_json::from_slice(&data[..]).unwrap(),
                            ),
                        }]
                    }
                }
            }
            OpType::EntryReply => {
                vec![BiliWebsocketInner {
                    header,
                    body: BiliWebsocketMessageBody::EntryReply,
                }]
            }

            OpType::HeartBeatReply => {
                let mut cursor = Cursor::new(data);
                let popularity = cursor.read_i32::<BigEndian>().unwrap_or(0);
                vec![BiliWebsocketInner {
                    header,
                    body: BiliWebsocketMessageBody::RoomPopularity(popularity),
                }]
            }

            _ => {
                // we currently don't deal with client-sent messages
                // but this could be useful if we'are gonna implement something
                // lika a mock BiliWebsocket Server
                warn!("Unexpected Op Type");
                vec![]
            }
        }
    }
}

// decompressed buffer contains one or more
// messages, we will extract them one by one
fn process_zlib_data(buf: Vec<u8>) -> Vec<BiliWebsocketInner> {
    let mut inners = vec![];

    let mut cur_buf = &buf[..];

    let mut offset = 0;
    let max_length = buf.len();

    while offset < max_length {
        // first parse current header
        let header = BiliWebsocketHeader::from_vec(cur_buf);
        let packet_length = header.packet_length;

        // this_buf: buffer for current inner
        // next_buf: the rest
        let (this_buf, next_buf) = cur_buf.split_at(packet_length as usize);

        // todo: get rid of this unwrap by returning a Result
        let this_inner = BiliWebsocketInner::from_binary(this_buf).unwrap();
        inners.push(this_inner);

        cur_buf = next_buf;

        offset += packet_length as usize;
    }

    inners
}

// header length is fixed 16
const HEADER_LENGTH: u16 = 16;
// don't know what's for, just 1
const SEQ: u32 = 1;
// Header Format:
// offset    length    type    endian    name           note
// 0         4         i32     Big       packet-length
// 4         2         i16     Big       header-length  Fixed 16
// 6         2         i16     Big       proto-version
// 8         4         i32     Big       Operation Type
// 12        4         i32     Big       Seq ID         Fixed 1
#[derive(Debug, PartialEq, Eq)]
pub struct BiliWebsocketHeader {
    packet_length: u32,
    header_length: u16,
    protocol_version: u16,
    op: OpType,
    seq: u32,
}

impl BiliWebsocketHeader {
    fn new(packet_length: u32, protocol_version: u16, op: OpType) -> Self {
        Self {
            packet_length,
            header_length: HEADER_LENGTH,
            protocol_version,
            op,
            seq: SEQ,
        }
    }

    /// read and parse a [BiliWebsocketHeader] from given byte array
    fn from_vec(buf: &[u8]) -> Self {
        // sanity check
        assert!(HEADER_LENGTH as usize <= buf.len());

        let mut cursor = Cursor::new(buf);

        let packet_length = cursor.read_u32::<BigEndian>().unwrap();
        let header_length = cursor.read_u16::<BigEndian>().unwrap();
        let protocol_version = cursor.read_u16::<BigEndian>().unwrap();
        let op: OpType = cursor.read_u32::<BigEndian>().unwrap().into();
        let seq = cursor.read_u32::<BigEndian>().unwrap();

        Self {
            packet_length,
            header_length,
            protocol_version,
            op,
            seq,
        }
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut header = vec![];
        // write packet length = header length + data length
        header.write_u32::<BigEndian>(self.packet_length).unwrap();
        // write header length
        header.write_u16::<BigEndian>(self.header_length).unwrap();
        // write proto-version 1
        header
            .write_u16::<BigEndian>(self.protocol_version)
            .unwrap();
        // write operatin type: enter room = 7
        header.write_u32::<BigEndian>(self.op.into()).unwrap();
        // write seq id
        header.write_u32::<BigEndian>(self.seq).unwrap();

        // sanity check
        assert_eq!(HEADER_LENGTH as usize, header.len());
        header
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpType {
    HeartBeat,
    HeartBeatReply,
    Notification,
    Entry,
    EntryReply,

    // place holder for all unknown ops
    Unknown,
}

impl From<OpType> for u32 {
    fn from(op: OpType) -> u32 {
        match op {
            OpType::HeartBeat => 2,
            OpType::HeartBeatReply => 3,
            OpType::Notification => 5,
            OpType::Entry => 7,
            OpType::EntryReply => 8,

            OpType::Unknown => 0,
        }
    }
}

impl From<u32> for OpType {
    fn from(value: u32) -> Self {
        match value {
            2 => OpType::HeartBeat,
            3 => OpType::HeartBeatReply,
            5 => OpType::Notification,
            7 => OpType::Entry,
            8 => OpType::EntryReply,

            _ => OpType::Unknown,
        }
    }
}

/// The first packet body sent to Bilibili Live
/// when websocket connection is established
#[derive(Debug, Serialize, Deserialize)]
struct FirstSecurityData {
    clientver: &'static str,
    platform: &'static str,
    protover: u64,
    uid: u64,
    roomid: i64,
    #[serde(rename = "type")]
    type_: u64,
}

impl FirstSecurityData {
    fn new(roomid: i64, uid: Option<u64>) -> Self {
        let uid = uid.unwrap_or(0);
        Self {
            clientver: "1.14.0",
            platform: "web",
            protover: 1,
            uid,
            roomid,
            type_: 2,
        }
    }
}

/// This struct is the inner message body of a websocket frame.
/// The reason why this is a separate type from [BiliWebsocketMessage]
/// is that [BiliWebsocketMessage] is the top level construct of communication protocol,
/// and the server might compress a vector of [BiliWebsocketInner] in a single frame it sends,
/// i.e., one [BiliWebsocketMessage] can parse to multiple [BiliWebsocketInner]s,
/// or it can contain only one [BiliWebsocketInner], which is tricky.
///
#[derive(Debug)]
pub struct BiliWebsocketInner {
    // inner has its own header
    header: BiliWebsocketHeader,
    body: BiliWebsocketMessageBody,
}

impl BiliWebsocketInner {
    pub fn get_op_type(&self) -> OpType {
        self.header.op
    }

    /// Consume the message and return the body
    pub fn into_body(self) -> BiliWebsocketMessageBody {
        self.body
    }
}

/// Message Body of a Bilibili's websocket message,
/// variants corresponding to different [OpType]
/// [OpType::HeartBeatReply] -> [BiliWebsocketMessageBody::RoomPopularity]
/// [OpType::Notification] -> [BiliWebsocketMessageBody::Notification]
/// [OpType::EntryReply] -> [BiliWebsocketMessageBody::EntryReply]
///
/// note: only the server-sent operations are represented here for furthr processing,
/// and client-sent operations are directly constructed
/// to the top-level [BiliWebsocketMessage] type
#[derive(Debug)]
pub enum BiliWebsocketMessageBody {
    // contains a single i32, which is the current
    // room popularity
    RoomPopularity(i32),
    // Notification
    Notification(NotificationBody),
    // entry reply contains no data
    EntryReply,
}

/// Notification Body of Bilibili's Websocket message,
/// This is basically all the interesting information about a live room
/// (Danmu, Gift, Subscription, etc).
pub type NotificationBody = serde_json::Value;

impl BiliWebsocketInner {
    fn from_binary(buf: &[u8]) -> DanmujiResult<Self> {
        // sanity check
        assert!(HEADER_LENGTH as usize <= buf.len());

        let (header, content) = buf.split_at(HEADER_LENGTH as usize);

        let header = BiliWebsocketHeader::from_vec(header);

        let body = match header.op {
            OpType::Notification => {
                let content_buf = if header.protocol_version == 2 {
                    // zlib
                    let mut z = ZlibDecoder::new(Cursor::new(content));
                    let mut decompressed_buf = vec![];
                    z.read_to_end(&mut decompressed_buf).unwrap();
                    decompressed_buf
                } else {
                    content.to_vec()
                };

                BiliWebsocketMessageBody::Notification(
                    serde_json::from_slice(&content_buf[..]).unwrap(),
                )
            }

            OpType::EntryReply => BiliWebsocketMessageBody::EntryReply,

            OpType::HeartBeatReply => {
                let mut cursor = Cursor::new(content);
                let popularity = cursor.read_i32::<BigEndian>().unwrap_or(0);

                BiliWebsocketMessageBody::RoomPopularity(popularity)
            }

            // currently don't deal with client-sent op types
            _ => unimplemented!(),
        };

        Ok(Self { header, body })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use byteorder::ReadBytesExt;

    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Eq)]
    struct TestJsonData {
        field1: u32,
        field2: String,
    }

    #[test]
    fn test_message_format() {
        let data = TestJsonData {
            field1: 0,
            field2: "Hello World".to_string(),
        };

        let test_bytes = serde_json::to_vec(&data).unwrap();
        let data_len = test_bytes.len();

        let msg = BiliWebsocketMessage::new(test_bytes.clone(), OpType::Entry, 1);

        let header = msg.header.to_vec();
        let mut cursor = Cursor::new(&header);

        // check header format
        // read packet length
        assert_eq!(
            data_len as u32 + HEADER_LENGTH as u32,
            cursor.read_u32::<BigEndian>().unwrap()
        );
        // read header length
        assert_eq!(HEADER_LENGTH, cursor.read_u16::<BigEndian>().unwrap());
        // read protocol version
        assert_eq!(1, cursor.read_u16::<BigEndian>().unwrap());
        // read operation type
        assert_eq!(
            OpType::Entry,
            cursor.read_u32::<BigEndian>().unwrap().into()
        );
        // read seq
        assert_eq!(SEQ, cursor.read_u32::<BigEndian>().unwrap());

        // check data format
        let data_bytes = msg.to_vec();

        // header match
        assert_eq!(&header[..], &data_bytes[..HEADER_LENGTH as usize],);

        // data match
        assert_eq!(&test_bytes[..], &data_bytes[HEADER_LENGTH as usize..],)
    }

    #[test]
    fn test_from_binary() {
        let data = TestJsonData {
            field1: 0,
            field2: "Hello World".to_string(),
        };

        let msg = BiliWebsocketMessage::new(serde_json::to_vec(&data).unwrap(), OpType::Entry, 2);

        let buf = msg.to_vec();
        let recovered_msg = BiliWebsocketMessage::from_binary(buf).unwrap();

        assert_eq!(msg, recovered_msg);
    }
}
