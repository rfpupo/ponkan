export type Profile = {
  id: string;
  name: string;
  updated_at: string;
  portable?: boolean;
  shared?: boolean;
  author?: string;
  description?: string;
};
export type PknPreview = {
  manifest: { name: string; author?: string; description?: string; updated_at: string };
  edited: boolean;
  duplicate: boolean;
  suggested_name: string;
  avatar?: string;
};
export type Modal = "create" | "rename" | null;
export type DesktopPrefs = {
  autostart: boolean;
  start_minimized: boolean;
  close_to_tray: boolean;
  retention_days: number;
  language: string;
};
export type MainPage = "home" | "profiles" | "trash" | "settings";
export type TrashEntry = {
  profile: Profile;
  deleted_at: string;
  avatar?: string | null;
  favorite: boolean;
};
export type TrashConfirmation = { ids: string[]; name?: string };
