import { mountStatEvaluationPlayer } from "./main.ts";

const root = document.querySelector("#app");
if (!(root instanceof HTMLElement)) {
  throw new Error("Missing #app mount element");
}

mountStatEvaluationPlayer(root);
