/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

// Tauri 返回的控件定义
export type ControlDef =
  | { type: "slider"; key: string; label: string; min: number; max: number; default: number }
  | { type: "select"; key: string; label: string; values: string[]; default: string }
  | { type: "number"; key: string; label: string; min: number; max: number; default: number };
