import { invoke } from "@tauri-apps/api/core";

export interface TreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: TreeNode[];
}

export const setVault = (path: string) => invoke<void>("set_vault", { path });
export const readTree = () => invoke<TreeNode[]>("read_tree");
export const readNote = (path: string) => invoke<string>("read_note", { path });
export const writeNote = (path: string, content: string) =>
  invoke<void>("write_note", { path, content });
export const createNote = (path: string) => invoke<string>("create_note", { path });
export const createFolder = (path: string) => invoke<void>("create_folder", { path });
export const movePath = (from: string, to: string) => invoke<void>("move_path", { from, to });
export const deletePath = (path: string) => invoke<void>("delete_path", { path });

// Panneau, fenêtres & raccourci global
export const hidePanel = () => invoke<void>("hide_panel");
export const showPanel = () => invoke<void>("show_panel_cmd");
export const openManager = () => invoke<void>("open_manager");
export const setShortcut = (toggle: string, newnote: string, widthPct: number) =>
  invoke<void>("set_shortcut", { toggle, newnote, widthPct });

// Git (local)
export const gitInit = () => invoke<void>("git_init");
export const gitCommit = () => invoke<boolean>("git_commit");
