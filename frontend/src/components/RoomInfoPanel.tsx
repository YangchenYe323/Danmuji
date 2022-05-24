import { useCallback, useState } from "react";
import { Room } from "../bindings/room";

type RoomInfoProp = {
	room: Room | null,
	connectToRoom: (arg0: string) => Promise<void>,
	disconnectFromRoom: () => Promise<void>,
}

const RoomInfoPanel = ({
	room,
	connectToRoom,
	disconnectFromRoom,
}: RoomInfoProp) => {
	const [roomId, setRoomId] = useState<string | undefined>(room?.roomid);

	const updateRoomId = useCallback((e: React.FormEvent<HTMLInputElement>) => {
		let roomId = (e.target as HTMLInputElement).value;
		console.log(roomId);
		setRoomId(roomId);
	}, [setRoomId]);

	const submitConnect = useCallback(() => {
		if (roomId !== undefined) {
			connectToRoom(roomId);
		}
	}, [roomId, connectToRoom]);

	// not connected to room
	if (room === null) {
		return (
			<div>
				<h1>
					你还没有连接到房间
				</h1>
				<input type="text" onInput={updateRoomId} placeholder="请输入房间号" ></input><br/>
				<button className="bg-blue-500 hover:bg-blue-700 text-white px-2 font-bold rounded mt-2" onClick={submitConnect}>连接</button>
			</div>
		);
	}

	// connected to room
	return(
		<div>
			<h1>
				当前连接到房间: {room.roomid}
			</h1>
			<button className="bg-blue-500 hover:bg-blue-700 text-white px-2 font-bold rounded mt-2" onClick={disconnectFromRoom}>断开连接</button>
		</div>
	)
}

export default RoomInfoPanel;