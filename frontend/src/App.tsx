import { Live } from "./Live";
import Config from "./Config";

export default function App() {
	return (
		<div className="flex flex-row flex-wrap justify-evenly items-stratch min-h-screen font-xiaowei">
			<Live />
			<Config />
		</div>
	);
}
