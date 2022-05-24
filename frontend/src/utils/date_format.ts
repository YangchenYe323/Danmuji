const padTo2Digits = num => {
	return num.toString().padStart(2, "0");
};

const formatDate = date => {
	return (
		[
			padTo2Digits(date.getHours()),
			padTo2Digits(date.getMinutes()),
			padTo2Digits(date.getSeconds()),
		].join(":")
	);
};

export default formatDate;
  