
const qrcode = async () => {
	const response = await fetch("http://127.0.0.1:8000/qrcode");
	return response.json();
};

const queryResult = async (qr) => {
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