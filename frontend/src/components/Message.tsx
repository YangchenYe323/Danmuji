import { BiliMessage } from "../bindings/BiliMessage";
import formatDate from "../utils/date_format";

declare interface MessageProp {
	message: BiliMessage;
}

const Message = ({ message }: MessageProp) => {
	console.log(message);
	if (message.type !== "Danmu") {
		return <div></div>;
	}

	//todo: handle other types of message with different component
	return (
		<div className="flex flex-wrap justify-items-start items-center animate-danmaku-movein font-mono h-8 min-h-fit">
			<span className="bg-gradient-to-r from-lime-400 to-yellow-200 text-xs text-white after:mr-1 shallow-shadowed-text">
				{formatDate(new Date(Number(message.body.sent_time)))}
			</span>
			{message.body.is_manager ? (
				<span className="border border-black bg-transparent rounded-sm text-xs text-emerald-500 font-serif mx-px">
					房
				</span>
			) : null}
			{message.body.guard !== "NoGuard" ? (
				<span className="border border-black bg-transparent rounded-md text-xs text-sky-900 font-serif px-1">
					{message.body.guard === "Governor"
						? "总督"
						: message.body.guard == "Admiral"
						? "提督"
						: "舰长"}
				</span>
			) : null}
			{message.body.medal ? (
				<div className="inline-flex text-xs text-white border rounded-md border-black min-h-fit mx-px bg-gradient-to-r from-cyan-500 to-blue-500">
					<span className="mx-px">
						{Number(message.body.medal.level)}
					</span>
					<span className="inline-block border-l mx-px my-0 py-0 border-black"></span>
					<span className="mx-px">{message.body.medal.name}</span>
				</div>
			) : null}
			<span className="after:content-[':'] text-cyan-200 shadowed-text mx-px">
				{message.body.uname}
			</span>
			<span className="text-white shadowed-text mx-px">
				{message.body.content}
			</span>
		</div>
	);
};

export default Message;
