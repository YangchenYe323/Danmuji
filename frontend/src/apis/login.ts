import { DanmujiApiResponse } from "../bindings/DanmujiApiResponse";
import { QrCode } from "../bindings/QrCode";
import { User } from "../bindings/user";

const qrcode = async (): Promise<DanmujiApiResponse<QrCode>> => {
	const response = await fetch("http://127.0.0.1:8000/qrcode");
	return response.json();
};

const queryResult = async (qr: QrCode): Promise<DanmujiApiResponse<User | null>> => {
	const response = await fetch("http://127.0.0.1:8000/loginCheck", {
		method: "POST",
		body: JSON.stringify(qr)
	});
	return response.json();
};

export {
	qrcode,
	queryResult,
};