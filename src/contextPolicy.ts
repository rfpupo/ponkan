import type { MainPage } from "./types";
export type Page = MainPage;
export type Kind = "page" | "profile" | "trash-item" | "selection" | "dialog" | "field";
export type Action =
  | "profiles"
  | "settings"
  | "trash"
  | "about"
  | "new"
  | "import"
  | "share"
  | "rename"
  | "delete"
  | "restore"
  | "purge"
  | "empty"
  | "copy"
  | "paste"
  | "cut"
  | "select-all";
export function contextActions(page: Page, kind: Kind, hasTrash: boolean): Action[] {
  if (kind === "profile") return ["share", "rename", "delete"];
  if (kind === "trash-item") return ["restore", "purge"];
  if (kind === "selection") return ["copy"];
  if (kind !== "page") return [];
  if (page === "home") return ["profiles", "settings", "trash", "about"];
  if (page === "profiles") return ["new", "import", "about"];
  if (page === "trash" && hasTrash) return ["empty", "about"];
  return ["about"];
}
export function textActions({
  length,
  start,
  end,
  editable,
}: {
  length: number;
  start: number;
  end: number;
  editable: boolean;
}): Action[] {
  const a: Action[] = [];
  if (end > start) a.push("copy");
  if (editable) a.push("paste");
  if (editable && end > start) a.push("cut");
  if (length > 0 && (start !== 0 || end !== length)) a.push("select-all");
  return a;
}
export function replaceSelection(
  value: string,
  start: number,
  end: number,
  text: string,
  maxLength: number,
) {
  const limit = maxLength < 0 ? text.length : Math.max(0, maxLength - value.length + (end - start));
  let insert = text.slice(0, limit);
  if (/[\uD800-\uDBFF]$/.test(insert)) insert = insert.slice(0, -1);
  return { value: value.slice(0, start) + insert + value.slice(end), caret: start + insert.length };
}
