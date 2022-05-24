import { useCallback, useEffect, useState } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { BiliMessage } from "./bindings/BiliMessage";
import MessageList from "./components/MessageList";
import {v4 as uuidv4} from "uuid";
import { Room } from "./bindings/room";
import { disconnect, getRoomStatus, roomInit } from "./apis/api";
import RoomInfoPanel from "./components/RoomInfoPanel";

// Wraps around BiliMessage with a uuid
// to aid react re-rendering
export type BiliUIMessage = {
	key: string,
	body: BiliMessage,
}

const connectToRoom = async (room_id: string): Promise<Room | null> => {
	const res = await roomInit(room_id);
	if (res.success) {
		return res.payload;
	}
	return null;
};

const queryConnectionStatus = async (): Promise<Room | null> => {
	const res = await getRoomStatus();
	if (res.success) {
		return res.payload;
	}
	return null;
};

const disconnectFromRoom = async (): Promise<boolean> => {
	const res = await disconnect();
	return res.success;
};

const baseUrl = "ws://0.0.0.0:9000/ws";
const connectionStates = {
	[ReadyState.CONNECTING]: "Connecting",
	[ReadyState.OPEN]: "Open",
	[ReadyState.CLOSING]: "Closing",
	[ReadyState.CLOSED]: "Closed",
	[ReadyState.UNINSTANTIATED]: "Uninstantiated",
};

export const Live = () => {
	// current room
	const [room, setRoom] = useState<Room | null>(null);
	const [socketUrl, setSocketUrl] = useState<string>("");
	const { sendMessage, lastMessage, readyState } = useWebSocket(socketUrl);

	const connect = useCallback(async (room_id: string): Promise<void> => {
		const res = await connectToRoom(room_id);
		if (res !== null) {
			setRoom(res);
		}
	}, [setRoom]);

	const disconnect = useCallback(async (): Promise<void> => {
		const disconnected = await disconnectFromRoom();
		if (disconnected) {
			setRoom(null);
		}
	}, [setRoom]);

	// query login status on amount
	useEffect(() => {
		const fetchRoom = async () => {
			const res = await queryConnectionStatus();
			console.log(res);
			setRoom(res);
		};

		fetchRoom();
	}, [setRoom]);

	// update socketUrl when room is updated
	useEffect(() => {
		if (room !== null) {
			console.log("Socket Url Changed");
			setSocketUrl(`${baseUrl}/${room.roomid}`);
		}
	}, [room]);

	// heartbeat
	useEffect(() => {
		const interval = setInterval(() => {
			sendMessage("Heartbeat");
		}, 20000);
		return () => clearInterval(interval);
	}, [sendMessage]);

	// parse BiliMessage
	let lastBiliMessage: BiliUIMessage | null = null;
	if (lastMessage !== null) {
		lastBiliMessage = {
			key: uuidv4(),
			body: JSON.parse(lastMessage.data),
		};
	}

	const status = connectionStates[readyState];
	return (
		<div className="basis-1/2 bg-gray-200">
			<RoomInfoPanel room={room} connectToRoom={connect} disconnectFromRoom={disconnect} />
			<h1 className="text-center">The WebSocket is currently {status}</h1>
			<MessageList newMessage={lastBiliMessage} />
		</div>
	);
};