/// Danmuji's API module
/// A direct correspondence with backend's interface

import { DanmujiApiResponse } from "../bindings/DanmujiApiResponse";
import { QrCode } from "../bindings/QrCode";
import { Room } from "../bindings/room";
import { User } from "../bindings/user";

const baseUrl = "http://0.0.0.0:9000";

const danmujiFetch = async <T>(
	url: string,
	method: "GET" | "POST" = "GET",
	data: string | undefined = undefined
): Promise<DanmujiApiResponse<T>> => {
	try {
		const response = await fetch(url, {
			method: method || "GET",
			body: data,
		});

		return await response.json();
	} catch (error) {
		console.log(error);
		return {
			success: false,
			payload: null,
		};
	}
};

/// get login qrcode
const qrcode = async (): Promise<DanmujiApiResponse<QrCode>> => {
	return await danmujiFetch<QrCode>(`${baseUrl}/qrcode`);
};

/// check login result
const queryResult = async (
	qr: QrCode
): Promise<DanmujiApiResponse<User | null>> => {
	return await danmujiFetch(
		`${baseUrl}/loginCheck`,
		"POST",
		JSON.stringify(qr)
	);
};

/// connect to room
const roomInit = async (roomId: string): Promise<DanmujiApiResponse<Room>> => {
	return await danmujiFetch<Room>(`${baseUrl}/roomInit/${roomId}`);
};

/// query currently connected room
const getRoomStatus = async (): Promise<DanmujiApiResponse<Room>> => {
	return await danmujiFetch<Room>(`${baseUrl}/roomStatus`);
};

const disconnect = async (): Promise<DanmujiApiResponse<void>> => {
	return await danmujiFetch(`${baseUrl}/disconnect`);
};

export { qrcode, queryResult, roomInit, getRoomStatus, disconnect };
