import { GiftMessage } from "../bindings/GiftMessage";

type GiftProp = {
	gift: GiftMessage;
};

const Gift = ({ gift }: GiftProp) => {
	return (
		<div className="flex flex-wrap h-full min-h-fit animate-danmaku-movein">
			<span>感谢</span>

			<span className="after:content-['投喂的']">{gift.uname}</span>

			<span>{`${gift.gift_num}个${gift.gift_name}`}</span>
		</div>
	);
};

export default Gift;
