import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { applyThemePreference, readThemePreference } from "./theme";
import "./styles.css";

applyThemePreference(readThemePreference());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
