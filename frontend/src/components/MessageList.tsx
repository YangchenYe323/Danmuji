import { useCallback, useEffect, useRef, useState } from "react";
import Message from "./Message";
import { BiliUIMessage, DanmujiUIConfig } from "../Live";
import { GiftMessage } from "../bindings/GiftMessage";

declare interface MessageListProp {
	newMessage: BiliUIMessage | null;
	config: DanmujiUIConfig;
}

const MessageList = ({ newMessage, config }: MessageListProp) => {
	// working message queue
	const [messageQueue, setMessageQueue] = useState<BiliUIMessage[]>([]);
	// gift combo map
	// saves uname-giftname -> GiftMessage
	// so that the same gift in a period can be accumulated and displayed once
	const [giftMap, setGiftMap] = useState<Map<string, BiliUIMessage>>(
		new Map()
	);
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
			// Case when top + height < containerTop:
			//
			// |--------------------|
			// |--------------------|  <- top + height
			// ----------------------  <- el.getBoundingClientRect().top
			// |					|
			// |					|
			// |					|
			// ----------------------
			const { top, height } = el.getBoundingClientRect();
			// this is the first visible child
			return top + height > containerTop;
		});
		childRefs.current.splice(0, remainingIndex);
		messageQueue.splice(0, remainingIndex);
	}, [messageQueue]);

	// handle new message
	useEffect(() => {
		if (newMessage !== null) {
			if (newMessage.body.type == "Gift" && config && config.giftCombo) {
				// handle gift combo
				// keep track of the same gift sent by the same user in the period
				const key = `${newMessage.body.body.uname}-${newMessage.body.body.gift_name}`;
				// we know that old will always be GiftMessage
				const old: GiftMessage | undefined = giftMap.get(key)?.body
					.body as GiftMessage;
				const old_num = old ? old.gift_num : 0;
				// console.log(old_num);
				// console.log(typeof old_num);
				// console.log(typeof newMessage.body.body.gift_num);
				giftMap.set(key, {
					key: newMessage.key,
					body: {
						type: "Gift",
						body: {
							...newMessage.body.body,
							gift_num: newMessage.body.body.gift_num + old_num,
						},
					},
				});

				if (old_num === 0) {
					// this is the first time we meet this gift message,
					// set up a timer to send it
					setTimeout(() => {
						const msg = giftMap.get(key);
						giftMap.delete(key);
						if (msg) {
							// console.log(msg);
							setMessageQueue((queue) => queue.concat(msg));
						}
					}, config.giftCombo * 1000);
				}
			} else {
				// we comes here if:
				// (a): message is not gift sending, so we don't do accumulation
				// (b): config is not set
				// directly update message queue and trigger re-rendering is fine
				setMessageQueue((queue) => queue.concat(newMessage));
			}
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
