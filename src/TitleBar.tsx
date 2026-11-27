import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, X } from "lucide-react";
import { toast } from "sonner";
import ponkanIcon from "./ponkan-icon.png";

import type { Translate } from "./i18n";

export function TitleBar({ t }: { t: Translate }) {
  const appWindow = getCurrentWindow();
  const report = (error: unknown) => toast.error(String(error));
  return (
    <header className="ponkan-titlebar">
      <div
        className="ponkan-title-drag"
        onMouseDown={(event) => {
          if (event.button === 0 && event.detail === 1) {
            event.preventDefault();
            void appWindow.startDragging().catch(report);
          }
        }}
      >
        <img src={ponkanIcon} alt="" draggable={false} />
        <span>pon//kan.</span>
      </div>
      <button
        type="button"
        className="title-control"
        aria-label={t("Minimizar")}
        title={t("Minimizar")}
        onClick={() => void appWindow.minimize().catch(report)}
      >
        <Minus size={17} />
      </button>
      <button
        type="button"
        className="title-control title-close"
        aria-label={t("Fechar")}
        title={t("Fechar")}
        onClick={() => void appWindow.close().catch(report)}
      >
        <X size={17} />
      </button>
    </header>
  );
}
