/// <reference types="vite/client" />

declare module "*.wasm?url" {
  const url: string;
  export default url;
}

declare module "../scripts/ensure-wasm-package.mjs" {
  export function ensureWasmPackageFresh(options: {
    force?: boolean;
    log?: (message: string) => void;
  }): Promise<void>;
  export function getWasmWatchTargets(): string[];
  export function isWasmSourcePath(filePath: string): boolean;
}
