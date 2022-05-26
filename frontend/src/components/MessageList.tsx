import { useCallback, useEffect, useRef, useState } from "react";
import Message from "./Message";
import { BiliUIMessage } from "../Live";

declare interface MessageListProp {
	newMessage: BiliUIMessage | null;
}

const MessageList = ({ newMessage }: MessageListProp) => {
	const [messageQueue, setMessageQueue] = useState<BiliUIMessage[]>([]);
	// reference to the container element
	const elementRef = useRef<HTMLDivElement>(null);
	// reference to each child element
	const childRefs = useRef<HTMLDivElement[]>([]);

	// scroll the dom element to bottom
	const scrollList = useCallback(() => {
		const element = elementRef.current;
		if (element !== null) {
			element.scrollTop = element.scrollHeight;
		}
	}, [elementRef]);

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

	// remove invisible child when messages change
	useEffect(() => {
		// this is the top of our container
		// ----------------------  <- el.getBoundingClientRect().top
		// |					|
		// |					|
		// |					|
		// ----------------------
		const containerTop = elementRef.current.getBoundingClientRect().top - 5;
		const remainingIndex = childRefs.current.findIndex((el) => {
			if (el === null) {
				// todo: why this could be null?
				return false;
			}
			// top + height is the buttom of a child message element
			// top + height > containerTop:
			//
			// |--------------------|
			// |--------------------|  <- top + height
			// ----------------------  <- el.getBoundingClientRect().top
			// |					|
			// |					|
			// |					|
			// ----------------------
			const { top, height } = el.getBoundingClientRect();
			return top + height > containerTop;
		});
		childRefs.current.splice(0, remainingIndex);
		messageQueue.splice(0, remainingIndex);
	}, [messageQueue]);

	// add message
	useEffect(() => {
		if (newMessage !== null) {
			setMessageQueue((queue) => queue.concat(newMessage));
		} else {
			// null message means a new connection is set, clear the screen
			setMessageQueue([]);
		}
	}, [newMessage]);

	return (
		<div
			className="grow bg-white h-96 overflow-hidden scroll-smooth"
			ref={elementRef}
		>
			{messageQueue.map((m, i) => (
				<Message
					key={m.key}
					message={m.body}
					ref={(rf) => {
						childRefs.current[i] = rf;
					}}
				/>
			))}
		</div>
	);
};

export default MessageList;
