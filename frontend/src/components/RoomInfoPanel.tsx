import { useCallback, useState } from "react";
import { Room } from "../bindings/room";
import Loading from "./svg/loading";

type RoomInfoProp = {
	room: Room | null;
	connectToRoom: (arg0: string) => Promise<void>;
	disconnectFromRoom: () => Promise<void>;
};

const RoomInfoPanel = ({
	room,
	connectToRoom,
	disconnectFromRoom,
}: RoomInfoProp) => {
	const [loading, setLoading] = useState<boolean>(false);
	const [roomId, setRoomId] = useState<string | undefined>(room?.roomid);

	const updateRoomId = useCallback(
		(e: React.FormEvent<HTMLInputElement>) => {
			const roomId = (e.target as HTMLInputElement).value;
			console.log(roomId);
			setRoomId(roomId);
		},
		[setRoomId]
	);

	const submitConnect = useCallback(async () => {
		if (roomId !== undefined) {
			// set loading
			setLoading(true);
			await connectToRoom(roomId);
			setLoading(false);
		}
	}, [roomId, connectToRoom, setLoading]);

	const submitDisconnect = useCallback(async () => {
		setLoading(true);
		await disconnectFromRoom();
		setLoading(false);
	}, [disconnectFromRoom, setLoading]);

	// have not connected to room
	if (room === null) {
		return (
			<div>
				{loading ? (
					<div>
						<Loading />
						<h1>正在连接</h1>
					</div>
				) : (
					<div>
						<h1>你还没有连接到房间</h1>
						<input
							type="text"
							onInput={updateRoomId}
							placeholder="请输入房间号"
						></input>
						<br />
						<button
							className="btn-primary mt-2"
							onClick={submitConnect}
						>
							连接
						</button>
					</div>
				)}
			</div>
		);
	}

	// have connected to room
	return (
		<div className="flex flex-wrap justify-start items-start h-fit border border-black">
			{loading ? (
				<div>
					<Loading />
					<h1>正在断开连接</h1>
				</div>
			) : (
				<div>
					<h1>当前连接到房间: {room.roomid}</h1>
					<h1>主播名称: {room.uname}</h1>
					<h1>直播标题: {room.content}</h1>
					<button
						className="btn-primary mt-1"
						onClick={submitDisconnect}
					>
						断开连接
					</button>
				</div>
			)}
		</div>
	);
};

export default RoomInfoPanel;
