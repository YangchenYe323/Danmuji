import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import App from "./App";
import Login from "./Login";
import "./tailwind.css";

const root = document.getElementById("root");
if (root !== null) {
	ReactDOM.createRoot(root).render(
		<BrowserRouter>
			<Routes>
				<Route path="/" element={<App />} />
				<Route path="login" element={<Login />} />
			</Routes>
		</BrowserRouter>
	);
}
