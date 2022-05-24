import { BiliMessage } from "../bindings/BiliMessage";
import formatDate from "../utils/date_format";

declare interface MessageProp {
	message: BiliMessage;
}

export function Message({ message }: MessageProp) {
	console.log(message);
	if (message.type !== "Danmu") {
		return <div></div>;
	}

	return (
		<div className="flex flex-wrap justify-items-start items-center animate-danmaku-movein font-mono h-8 min-h-fit">
			<span className="bg-lime-300 text-xs after:mr-1">
				{formatDate(new Date(Number(message.body.sent_time)))}
			</span>
			{message.body.is_manager ? (
				<span className="border border-black bg-transparent rounded-sm text-xs text-emerald-500 font-serif mx-px">
					房
				</span>
			) : null}
			{message.body.guard !== "NoGuard" ? (
				<span className="border border-black bg-transparent rounded-md text-xs text-sky-900 font-serif mx-px">
					{message.body.guard === "Governor"
						? "总"
						: message.body.guard == "Admiral"
						? "提"
						: "舰"}
				</span>
			) : null}
			{message.body.medal ? (
				<div className="inline-flex text-xs text-white border rounded-md border-black min-h-fit mx-px bg-gradient-to-r from-cyan-500 to-blue-500">
					<span className="mx-px">{Number(message.body.medal.level)}</span>
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
}
