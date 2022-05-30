import { useCallback, useEffect, useState } from "react";
import { getGiftConfig, setGiftConfig } from "../apis/api";
import { GiftThankConfig } from "../bindings/GiftThankConfig";

const placeholder_text = `示例模版：
感谢{ uname }送来的{ gift_num }个{ gift_name }
目前可用的礼物macro:
{uname}: 送礼用户名
{gift_num}: 礼物个数
{gift_name}: 礼物名称
`;

const GiftSettingsPanel = () => {
	const [config, setConfig] = useState<GiftThankConfig>(null);

	useEffect(() => {
		const queryGiftConfig = async () => {
			const res = await getGiftConfig();
			if (res.payload !== null) {
				setConfig(res.payload);
			}
		};
		queryGiftConfig();
	}, []);

	const submitSettingChange = useCallback(
		async (config: GiftThankConfig) => {
			const res = await setGiftConfig(config);
			if (res.success) {
				alert("修改成功");
				setConfig(config);
			} else {
				alert("修改失败");
			}
		},
		[setConfig]
	);

	return (
		<div className="self-stretch border border-cyan-400 p-2">
			<h1 className="shadowed-text text-cyan-100 text-xl">
				礼物感谢姬设置:
			</h1>
			<form
				onSubmit={async (e) => {
					e.preventDefault();
					// open or not
					const checkbox: HTMLInputElement = (
						e.target as HTMLFormElement
					).querySelector("#open");
					const open = checkbox.checked;

					// thank msg template
					const templatebox: HTMLTextAreaElement = (
						e.target as HTMLFormElement
					).querySelector("#template");

					let template = templatebox.value;
					if (!template && config) {
						template = config.template;
					}

					const newConfig: GiftThankConfig = {
						open,
						template,
					};

					await submitSettingChange(newConfig);
				}}
			>
				<label
					className="text-cyan-100 shadowed-text mr-2"
					htmlFor="open"
				>
					打开
				</label>
				<input type="checkbox" id="open" />

				<p className="text-md before:content-['('] after:content-[')']">
					当前：{config && config.open ? "打开" : "关闭"}
				</p>
				<br />

				<label
					className="text-cyan-100 shadowed-text mr-2"
					htmlFor="template"
				>
					感谢弹幕模版:{" "}
				</label>
				<p className="text-md before:content-['('] after:content-[')']">
					当前模版：{config ? config.template : "未设置"}
				</p>
				<br />
				<textarea
					id="template"
					name="template"
					rows={6}
					cols={60}
					placeholder={placeholder_text}
				></textarea>
				<br />
				<button className="btn-primary" value="submit">
					提交设置
				</button>
			</form>
		</div>
	);
};

export default GiftSettingsPanel;
