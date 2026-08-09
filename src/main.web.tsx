import ReactDOM from "react-dom/client";
import AppWeb from "./AppWeb";
import "./styles/globals.css";

const savedTheme = localStorage.getItem("samsmrti-theme");
if (savedTheme === "dark" || (!savedTheme && window.matchMedia("(prefers-color-scheme: dark)").matches)) {
  document.documentElement.classList.add("dark");
}

// No React.StrictMode here (unlike desktop's main.tsx): its dev-only
// double-invoke of ReviewSessionPage's mount effect races two
// startSession()/endSession() calls against the Worker's async round-trip
// and can leave the review queue desynced from the DB in local dev. Confirmed
// harmless for real users -- StrictMode's double-invoke only happens in dev,
// never in a `vite build` production bundle -- so this only affects testing
// this target locally, not the shipped PWA.
ReactDOM.createRoot(document.getElementById("root")!).render(<AppWeb />);
