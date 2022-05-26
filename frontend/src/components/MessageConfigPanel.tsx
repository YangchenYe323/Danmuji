import { DanmujiUIConfig } from "../Live";

type ConfigPanelProp = {
	config: DanmujiUIConfig;
	submitConfig: (DanmujiUIConfig) => void;
};

const giftComboValues = [0, 1, 2, 3];

const MessageConfigPanel = ({ config, submitConfig }: ConfigPanelProp) => {
	return (
		<div className="h-screen min-h-fit border border-black my-2">
			<h1 className="text-2xl text-center shadowed-text text-emerald-200">
				弹幕姬显示设置
			</h1>
			<form
				onSubmit={(e) => {
					e.preventDefault();
					// extract gift combo selection
					const select: HTMLSelectElement = (
						e.target as HTMLFormElement
					).querySelector("#giftCombo");
					const index = select.selectedIndex;
					const giftComboValue = select.options[index].value;

					submitConfig({
						...config,
						giftCombo:
							giftComboValue === "0"
								? undefined
								: Number(giftComboValue),
					});
				}}
			>
				<label htmlFor="giftCombo">礼物延迟显示设置:</label>
				<select name="giftCombo" id="giftCombo">
					{/* <option value={undefined}>不延迟</option>
					<option value={1}>1秒</option>
					<option value={2}>2秒</option>
					<option value={3}>3秒</option> */}
					{giftComboValues.map((val, i) => (
						<option
							key={i}
							value={val}
							selected={config.giftCombo === val}
						>
							{val === 0 ? "不延迟" : `${val}秒`}
						</option>
					))}
				</select>

				<br />
				<input className="btn-primary" type="submit" value="提交修改" />
			</form>
		</div>
	);
};

export default MessageConfigPanel;
