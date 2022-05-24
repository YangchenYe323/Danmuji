import type { DanmuMessage } from "./DanmuMessage";

export type BiliMessage = { type: "Danmu", body: DanmuMessage } | { type: "RoomPopularity", body: number };