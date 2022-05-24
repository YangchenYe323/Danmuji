import React from "react";
import { BiliMessage } from "../bindings/BiliMessage";
import formatDate from "../utils/date_format";

declare interface MessageProp {
	message: BiliMessage,
}

export function Message({
	message,
}: MessageProp) {

	if (message.type !== "Danmu") {
		return (
			<div></div>
		);
	}

	return (
		<div className="animate-danmaku-movein">
			<span className="bg-lime-300 text-xs after:mr-1">
				{formatDate(new Date(Number(message.body.sent_time)))}
			</span>
			{
				message.body.is_manager? <span className="border-2 border-black bg-white rounded-lg text-sm">房</span> : null
			}
			<span>{message.body.uname}</span>	
			:
			<span>{message.body.content}</span>
		</div>
	);
}