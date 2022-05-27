import { useEffect, useState } from "react";
import QrCodePanel from "react-qr-code";
import { QrCode } from "./bindings/QrCode";
import { qrcode as getQrCode, queryResult } from "./apis/api";
import Loading from "./components/svg/loading";

const Login = () => {
	const [qrLoading, setQrLoading] = useState<boolean>(true);
	const [qrcode, setQrCode] = useState<QrCode | null>(null);

	// fetch qrcode
	useEffect(() => {
		const fetchQrCode = async () => {
			const res = await getQrCode();
			if (res.success) {
				setQrCode(res.payload);
				setQrLoading(false);
			} else {
				setTimeout(fetchQrCode, 1000);
			}
		};
		fetchQrCode();
	}, []);

	// check login status
	useEffect(() => {
		const checkLoginStatus = async () => {
			const res = await queryResult(qrcode);
			if (res.success) {
				alert("登录成功");
				window.location.href = "../";
			} else {
				setTimeout(checkLoginStatus, 500);
			}
		};
		if (qrcode !== null) {
			checkLoginStatus();
		}
	}, [qrcode]);

	return (
		<div>
			{qrLoading ? (
				<div>
					<h1>正在获取登录二维码</h1>
					<Loading />
				</div>
			) : (
				<div>
					<h1>请扫描下方二维码登录:</h1>
					<QrCodePanel value={qrcode.url} />
				</div>
			)}
		</div>
	);
};

export default Login;
