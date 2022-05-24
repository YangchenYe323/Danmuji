import { useEffect } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { BiliMessage } from "./bindings/BiliMessage";
import MessageList from "./components/MessageList";
import {v4 as uuidv4} from 'uuid';

// Wraps around BiliMessage with a uuid
// to aid react re-rendering
export type BiliUIMessage = {
	key: string,
	body: BiliMessage,
}

const socketUrl = "ws://0.0.0.0:9000/ws";
const connectionStates = {
	[ReadyState.CONNECTING]: "Connecting",
	[ReadyState.OPEN]: "Open",
	[ReadyState.CLOSING]: "Closing",
	[ReadyState.CLOSED]: "Closed",
	[ReadyState.UNINSTANTIATED]: "Uninstantiated",
};

export const Live = () => {
	const { sendMessage, lastMessage, readyState } = useWebSocket(socketUrl);

	// parse BiliMessage
	let lastBiliMessage: BiliUIMessage | null = null;
	if (lastMessage !== null) {
		lastBiliMessage = {
			key: uuidv4(),
			body: JSON.parse(lastMessage.data),
		};
	}

	// heartbeat
	useEffect(() => {
		const interval = setInterval(() => {
			sendMessage("Heartbeat");
		}, 5000);
		return () => clearInterval(interval);
	}, [sendMessage]);

	const status = connectionStates[readyState];
	return (
		<div className="basis-2/5 bg-gray-200">
			<h1 className="text-center">The WebSocket is currently {status}</h1>
			<MessageList newMessage={lastBiliMessage} />
		</div>
	);
};