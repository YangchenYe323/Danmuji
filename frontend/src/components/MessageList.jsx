import { Message } from "./Message"

export default function({
	messages
}) {

	return (
		<div>
			{messages.map((m) => 
				<Message message = {m}/>
			)}
		</div>
	)
}