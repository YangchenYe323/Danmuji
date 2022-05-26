import type { DanmuMessage } from "./DanmuMessage";
import type { GiftMessage } from "./GiftMessage";

export type BiliMessage =
	| { type: "Danmu"; body: DanmuMessage }
	| { type: "Gift"; body: GiftMessage }
	| { type: "RoomPopularity"; body: number };
