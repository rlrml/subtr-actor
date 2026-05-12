const root = document.querySelector("#app");
if (!(root instanceof HTMLElement)) {
  throw new Error("Missing #app mount element");
}

const { mountStatEvaluationPlayer } = await import("./main.ts");
mountStatEvaluationPlayer(root);

export {};
