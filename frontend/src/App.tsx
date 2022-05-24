import { useState } from "react";
import { qrcode, queryResult } from "./apis/api";
import QRCode from "react-qr-code";
import { Live } from "./Live";
import { QrCode } from "./bindings/QrCode";

export default function App() {
	const [code, setQrcode] = useState<QrCode>({
		url: "",
		oauthKey: "",
	});

	const get = async () => {
		const qr = await qrcode();
		if (qr.payload != null) {
			setQrcode(qr.payload);
		}
	};

	const query = async () => {
		const res = await queryResult(code);
		console.log(res);
	};

	return (
		<div className="flex flex-row flex-wrap justify-evenly items-stratch min-h-screen">
			<Live />

			<div className="basis-1/2 bg-gray-200">
				<QRCode value={code.url} onClick={get} />
				<button
					className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded m-8"
					onClick={query}
				>
					Check Result
				</button>
			</div>
		</div>
	);
}
