import { useEffect, useState } from "react";
import { getUser, logoutUser } from "./apis/api";
import { User } from "./bindings/User";
import { Link } from "react-router-dom";
import GiftSettingsPanel from "./components/GiftSettingsPanel";

const queryUser = async (): Promise<User | null> => {
	const res = await getUser();
	return res.payload;
};

const Config = () => {
	const [user, setUser] = useState<User | null>(null);

	// query user on mount
	useEffect(() => {
		const updateLoginStatus = async () => {
			const user = await queryUser();
			if (user !== null) {
				setUser(user);
			}
		};
		updateLoginStatus();
	}, []);

	const logout = async () => {
		await logoutUser();
		setUser(null);
	};

	return (
		<div className="basis-1/2 bg-gray-200 border border-black">
			{user !== null ? (
				<div className="flex flex-col flex-wrap">
					<div className="self-center text-center">
						<h1 className="text-center text-2xl">
							用户: {user.uname}
						</h1>
						<button
							className="block self-center btn-primary "
							onClick={logout}
						>
							断开登录
						</button>
					</div>
					<GiftSettingsPanel />
				</div>
			) : (
				<div>
					<h1 className="inline text-2xl">您还没有登录: </h1>
					<Link className="btn-primary" to="/login">
						去登录
					</Link>
				</div>
			)}
		</div>
	);
};

export default Config;
