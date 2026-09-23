import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
// Bundled, not fetched: the app has to look right with the network off.
import "@fontsource/caprasimo/400.css";
import "@fontsource/figtree/400.css";
import "@fontsource/figtree/600.css";
import "@fontsource/figtree/700.css";
import "./styles/tokens.css";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
