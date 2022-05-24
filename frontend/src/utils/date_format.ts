const padTo2Digits = (num: Number) => {
	return num.toString().padStart(2, "0");
};

const formatDate = (date: Date) => {
	return (
		[
			padTo2Digits(date.getHours()),
			padTo2Digits(date.getMinutes()),
			padTo2Digits(date.getSeconds()),
		].join(":")
	);
};

export default formatDate;
  