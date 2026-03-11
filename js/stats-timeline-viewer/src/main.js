import "./styles.css";
import { StatsTimelineViewerApp } from "./app.js";

const app = new StatsTimelineViewerApp(document.getElementById("app"));
app.initialize().catch((error) => {
  console.error("Failed to initialize stats timeline viewer:", error);
});
