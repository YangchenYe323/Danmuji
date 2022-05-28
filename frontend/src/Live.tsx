import { useCallback, useEffect, useState } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { BiliMessage } from "./bindings/BiliMessage";
import MessageList from "./components/MessageList";
import { v4 as uuidv4 } from "uuid";
import { Room } from "./bindings/room";
import { disconnect, getRoomStatus, roomInit } from "./apis/api";
import RoomInfoPanel from "./components/RoomInfoPanel";
import MessageConfigPanel from "./components/MessageConfigPanel";

// Wraps around BiliMessage with a uuid
// to aid react re-rendering
export type BiliUIMessage = {
	key: string;
	body: BiliMessage;
};

// Danmuji Display Configuration
export type DanmujiUIConfig = {
	// accumulate gift sent in the given period and
	// show in one batch
	giftCombo: number | undefined;
};

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

const connectionStates = {
	[ReadyState.CONNECTING]: "连接中",
	[ReadyState.OPEN]: "开启",
	[ReadyState.CLOSING]: "关闭中",
	[ReadyState.CLOSED]: "关闭",
	[ReadyState.UNINSTANTIATED]: "未初始化",
};

export const Live = () => {
	// current room config
	const [room, setRoom] = useState<Room | null>(null);
	// websocket state
	const { sendMessage, lastMessage, readyState } = useWebSocket(
		// this will be proxied by vite to the right url
		`ws://${window.location.host}/api/ws`
	);
	// last Bili UI message
	const [lastUIMessage, setLastUIMessage] = useState<BiliUIMessage | null>(
		null
	);
	// danmujin configuration state
	const [config, setConfig] = useState<DanmujiUIConfig>({
		giftCombo: undefined,
	});

	const connect = useCallback(
		async (room_id: string): Promise<void> => {
			const res = await connectToRoom(room_id);
			if (res !== null) {
				setRoom(res);
			}
		},
		[setRoom]
	);

	const disconnect = useCallback(async (): Promise<void> => {
		const disconnected = await disconnectFromRoom();
		if (disconnected) {
			setRoom(null);
		}
	}, [setRoom]);

	const submitConfig = (config: DanmujiUIConfig) => {
		setConfig(config);
	};

	// query login status on amount
	useEffect(() => {
		const fetchRoom = async () => {
			const res = await queryConnectionStatus();
			console.log(`Connected to Room: ${res}`);
			setRoom(res);
		};

		fetchRoom();
	}, [setRoom]);

	// handle heartbeat
	useEffect(() => {
		// send heartbeat to keep connection alive
		// every 20 seconds
		const interval = setInterval(() => {
			sendMessage("Heartbeat");
		}, 20000);
		return () => clearInterval(interval);
	}, [sendMessage]);

	// console.log(`Danmuji Config: `);
	// console.log(config);

	// parse websocket message
	useEffect(() => {
		let lastBiliMessage = null;
		if (lastMessage !== null) {
			lastBiliMessage = {
				key: uuidv4(),
				body: JSON.parse(lastMessage.data),
			};
		}
		setLastUIMessage(lastBiliMessage);
	}, [lastMessage]);

	const status = connectionStates[readyState];
	return (
		<div className="basis-1/2 bg-gray-200">
			<RoomInfoPanel
				room={room}
				connectToRoom={connect}
				disconnectFromRoom={disconnect}
			/>
			<h1 className="text-center shadowed-text text-emerald-200">
				当前Websocket订阅状态：{status}
			</h1>
			<MessageList newMessage={lastUIMessage} config={config} />
			<MessageConfigPanel config={config} submitConfig={submitConfig} />
		</div>
	);
};
