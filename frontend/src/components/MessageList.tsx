import { useCallback, useEffect, useRef, useState } from "react";
import { Message } from "./Message";
import { BiliUIMessage } from "../Live";

declare interface MessageListProp {
	newMessage: BiliUIMessage | null;
}

const MessageList = ({ newMessage }: MessageListProp) => {
	const [messageQueue, setMessageQueue] = useState<BiliUIMessage[]>([]);
	const elementRef = useRef<HTMLDivElement>(null);

	// scroll the dom element to bottom
	const scrollList = useCallback(() => {
		const element = elementRef.current;
		if (element !== null) {
			element.scrollTop = element.scrollHeight;
		}
	}, []);

	// set up event listener
	useEffect(() => {
		window.addEventListener("resize", scrollList);
		return () => {
			window.removeEventListener("resize", scrollList);
		};
	}, []);

	// scroll to bottom when messages change
	useEffect(() => {
		scrollList();
	}, [messageQueue]);

	// add message
	useEffect(() => {
		if (newMessage !== null) {
			setMessageQueue((queue) => queue.concat(newMessage));
		} else {
			setMessageQueue([]);
		}
	}, [newMessage]);

	return (
		<div className="grow bg-cyan-100 h-96 ">
			<div
				className="grow bg-white h-96 overflow-hidden scroll-smooth"
				ref={elementRef}
			>
				{messageQueue.map((m) => (
					<Message key={m.key} message={m.body} />
				))}
			</div>
		</div>
	);
};

export default MessageList;
