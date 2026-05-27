export function el<K extends keyof HTMLElementTagNameMap>(
  tagName: K,
  options: { className?: string; text?: string; id?: string } = {},
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tagName);
  if (options.className) element.className = options.className;
  if (options.id) element.id = options.id;
  if (options.text !== undefined) element.textContent = options.text;
  return element;
}
