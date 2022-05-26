import { DanmuMessage } from "../bindings/DanmuMessage";
import formatDate from "../utils/date_format";

type DanmuProp = {
	danmu: DanmuMessage;
};

/// 弹幕消息
const Danmu = ({ danmu }: DanmuProp) => {
	return (
		<div className="flex flex-wrap justify-items-start items-center animate-danmaku-movein h-full min-h-fit">
			<span className="bg-gradient-to-r from-lime-400 to-yellow-200 text-xs text-white after:mr-1 shallow-shadowed-text">
				{formatDate(new Date(Number(danmu.sent_time)))}
			</span>
			{danmu.is_manager ? (
				<span className="border border-black bg-transparent rounded-sm text-xs text-emerald-500 mx-px">
					房
				</span>
			) : null}
			{danmu.guard !== "NoGuard" ? (
				<span className="border border-black bg-transparent rounded-md text-xs text-sky-900 px-1">
					{danmu.guard === "Governor"
						? "总督"
						: danmu.guard == "Admiral"
						? "提督"
						: "舰长"}
				</span>
			) : null}
			{danmu.medal ? (
				<div className="inline-flex text-xs text-white border rounded-md border-black min-h-fit mx-px bg-gradient-to-r from-cyan-500 to-blue-500">
					<span className="mx-px">{Number(danmu.medal.level)}</span>
					<span className="inline-block border-l mx-px my-0 py-0 border-black"></span>
					<span className="mx-px">{danmu.medal.name}</span>
				</div>
			) : null}
			<span className="after:content-[':'] text-cyan-200 shadowed-text mx-px">
				{danmu.uname}
			</span>
			<span className="text-white shadowed-text mx-px">
				{danmu.content}
			</span>
		</div>
	);
};

export default Danmu;
