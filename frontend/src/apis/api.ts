import { DanmujiApiResponse } from "../bindings/DanmujiApiResponse";
import { QrCode } from "../bindings/QrCode";
import { Room } from "../bindings/room";
import { User } from "../bindings/user";

const baseUrl = "http://0.0.0.0:9000";

/// get login qrcode
const qrcode = async (): Promise<DanmujiApiResponse<QrCode>> => {
	const response = await fetch(`${baseUrl}/qrcode`);
	return await response.json();
};

/// check login result
const queryResult = async (qr: QrCode): Promise<DanmujiApiResponse<User | null>> => {
	const response = await fetch(`${baseUrl}/loginCheck`, {
		method: "POST",
		body: JSON.stringify(qr)
	});
	return await response.json();
};

/// connect to room
const roomInit = async (roomId: string): Promise<DanmujiApiResponse<Room | null>> => {
	const response = await fetch(`${baseUrl}/roomInit/${roomId}`);
	return await response.json();
}; 

/// query currently connected room
const getRoomStatus = async (): Promise<DanmujiApiResponse<Room | null>> => {
	const response = await fetch(`${baseUrl}/roomStatus`);
	return await response.json();
};

const disconnect = async (): Promise<DanmujiApiResponse<void>> => {
	const response = await fetch(`${baseUrl}/disconnect/1234`);
	return await response.json();
};

export {
	qrcode,
	queryResult,
	roomInit,
	getRoomStatus,
	disconnect,
};