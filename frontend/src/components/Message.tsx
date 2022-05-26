import React, { ForwardedRef } from "react";
import { BiliMessage } from "../bindings/BiliMessage";
import Danmu from "./Danmu";
import Gift from "./Gift";

declare interface MessageProp {
	message: BiliMessage;
}

const Message = React.forwardRef<HTMLDivElement, MessageProp>(function Message(
	{ message }: MessageProp,
	ref: ForwardedRef<HTMLDivElement>
) {
	// console.log(message);

	//todo: handle other types of message with different component
	return (
		<div ref={ref}>
			{message.type === "Danmu" ? (
				<Danmu danmu={message.body} />
			) : message.type === "Gift" ? (
				<Gift gift={message.body} />
			) : null}
		</div>
	);
});

export default Message;
