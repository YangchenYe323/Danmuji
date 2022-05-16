import {Message} from "./components/Message.jsx"
import MessageList from "./components/MessageList.jsx"
import { useState } from "react"
import {qrcode, queryResult} from "./apis/login.jsx"
import QRCode from "react-qr-code";

export default function App() {
  
  const [code, setQrcode] = useState({
    url: "",
    oauthKey: "",
  })

  const get = async () => {
    let qr = await qrcode();
    console.log(qr)
    setQrcode(qr);
  }

  const query = async () => {
    let res = await queryResult(code)
    console.log(res)
  }

  const danmus = [
    "第一条弹幕",
    "第二条弹幕",
    "第三条弹幕",
  ]

	return (
    <div>
      <div class="flex">
        <span class="animate-waving-hand">👋🏻</span>
      </div>

      
      <MessageList messages={danmus}/>

      <QRCode value={code.url} onClick={get}/>

      <button onClick={query}>Check Result</button>

    </div>
	)
}
