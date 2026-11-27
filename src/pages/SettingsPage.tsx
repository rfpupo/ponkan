import { Moon, Sun, Monitor, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { SettingsSwitch } from "@/components/shadcn-studio/switch/switch-14";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import flagBr from "flag-icons/flags/4x3/br.svg";
import flagEs from "flag-icons/flags/4x3/es.svg";
import flagUs from "flag-icons/flags/4x3/us.svg";
import type { Language, Translate } from "../i18n";
import type { DesktopPrefs } from "../types";
type Props = {
  t: Translate;
  theme: string;
  setTheme: (value: string) => void;
  language: Language;
  setLanguage: (value: Language) => void;
  confirmLoad: boolean;
  setConfirmLoad: (value: boolean) => void;
  confirmSave: boolean;
  setConfirmSave: (value: boolean) => void;
  autoBackup: boolean;
  setAutoBackup: (value: boolean) => void;
  desktop: DesktopPrefs | null;
  desktopBusy: boolean;
  updateDesktop: (key: string, value: boolean | number | string) => Promise<void>;
  openTrash: (from: "settings") => void;
};
export function SettingsPage({
  t,
  theme,
  setTheme,
  language,
  setLanguage,
  confirmLoad,
  setConfirmLoad,
  confirmSave,
  setConfirmSave,
  autoBackup,
  setAutoBackup,
  desktop,
  desktopBusy,
  updateDesktop,
  openTrash,
}: Props) {
  return (
    <section className="ponkan-settings">
      <section className="ponkan-setting-group">
        <h2>{t("Geral")}</h2>
        <div className="ponkan-setting-card">
          <div className="ponkan-setting-row">
            <span id="theme-label" className="ponkan-select-label">
              {t("Aparência")}
            </span>
            <Select value={theme} onValueChange={setTheme}>
              <SelectTrigger
                id="theme"
                aria-labelledby="theme-label"
                className="ponkan-setting-select"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="dark">
                  <Moon size={16} aria-hidden="true" />
                  {t("Escuro")}
                </SelectItem>
                <SelectItem value="light">
                  <Sun size={16} aria-hidden="true" />
                  {t("Claro")}
                </SelectItem>
                <SelectItem value="system">
                  <Monitor size={16} aria-hidden="true" />
                  {t("Sistema")}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="ponkan-setting-row">
            <span id="language-label" className="ponkan-select-label">
              {t("Idioma")}
            </span>
            <Select value={language} onValueChange={(value) => setLanguage(value as Language)}>
              <SelectTrigger
                id="language"
                aria-labelledby="language-label"
                className="ponkan-setting-select"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="pt-BR">
                  <img className="ponkan-flag" src={flagBr} alt="" /> Português
                </SelectItem>
                <SelectItem value="es-ES">
                  <img className="ponkan-flag" src={flagEs} alt="" /> Español
                </SelectItem>
                <SelectItem value="en-US">
                  <img className="ponkan-flag" src={flagUs} alt="" /> English
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </section>
      <section className="ponkan-setting-group">
        <h2>{t("Segurança")}</h2>
        <div className="ponkan-setting-card">
          <div className="ponkan-setting-row">
            <label htmlFor="confirm-load">{t("Confirmar antes de carregar")}</label>
            <SettingsSwitch
              yes={t("Sim")}
              no={t("Não")}
              id="confirm-load"
              checked={confirmLoad}
              onCheckedChange={setConfirmLoad}
            />
          </div>
          <div className="ponkan-setting-row">
            <label htmlFor="confirm-save">{t("Confirmar antes de sobrescrever")}</label>
            <SettingsSwitch
              yes={t("Sim")}
              no={t("Não")}
              id="confirm-save"
              checked={confirmSave}
              onCheckedChange={setConfirmSave}
            />
          </div>
          <div className="ponkan-setting-row">
            <div>
              <label htmlFor="auto-backup">{t("Backup automático")}</label>
              <p id="backup-description">
                {t("Cria uma cópia das configurações atuais antes de carregar outro perfil.")}
              </p>
            </div>
            <SettingsSwitch
              yes={t("Sim")}
              no={t("Não")}
              id="auto-backup"
              aria-describedby="backup-description"
              checked={autoBackup}
              onCheckedChange={setAutoBackup}
            />
          </div>
        </div>
      </section>
      <section className="ponkan-setting-group">
        <h2>{t("Comportamento")}</h2>
        <div className="ponkan-setting-card">
          <div className="ponkan-setting-row">
            <label htmlFor="autostart">{t("Iniciar com o Windows")}</label>
            <SettingsSwitch
              yes={t("Sim")}
              no={t("Não")}
              id="autostart"
              disabled={!desktop || desktopBusy}
              checked={desktop?.autostart ?? false}
              onCheckedChange={(v) => void updateDesktop("autostart", v)}
            />
          </div>
          <div className="ponkan-setting-row">
            <label htmlFor="start-minimized">{t("Iniciar minimizado na bandeja")}</label>
            <SettingsSwitch
              yes={t("Sim")}
              no={t("Não")}
              id="start-minimized"
              disabled={!desktop || desktopBusy}
              checked={desktop?.start_minimized ?? true}
              onCheckedChange={(v) => void updateDesktop("start_minimized", v)}
            />
          </div>
          <div className="ponkan-setting-row">
            <label htmlFor="close-to-tray">{t("Continuar em segundo plano ao fechar")}</label>
            <SettingsSwitch
              yes={t("Sim")}
              no={t("Não")}
              id="close-to-tray"
              disabled={!desktop || desktopBusy}
              checked={desktop?.close_to_tray ?? true}
              onCheckedChange={(v) => void updateDesktop("close_to_tray", v)}
            />
          </div>
        </div>
      </section>
      <section className="ponkan-setting-group">
        <h2>{t("Armazenamento")}</h2>
        <div className="ponkan-setting-card">
          <div className="ponkan-setting-row">
            <span>{t("Lixeira")}</span>
            <Button
              variant="outline"
              className="ponkan-settings-trash-action"
              onClick={() => openTrash("settings")}
            >
              <Trash2 size={16} />
              {t("Abrir lixeira")}
            </Button>
          </div>
          <div className="ponkan-setting-row">
            <div>
              <span id="retention-label" className="ponkan-select-label">
                {t("Limpar lixeira automaticamente")}
              </span>
              <p>{t("Prazo contado a partir da exclusão de cada perfil.")}</p>
            </div>
            <Select
              disabled={!desktop || desktopBusy}
              value={String(desktop?.retention_days ?? 7)}
              onValueChange={(v) => void updateDesktop("retention_days", Number(v))}
            >
              <SelectTrigger className="ponkan-setting-select" aria-labelledby="retention-label">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {[0, 1, 7, 15, 30, 60, 90].map((n) => (
                  <SelectItem key={n} value={String(n)}>
                    {n === 0 ? t("Nunca") : n === 1 ? t("1 dia") : n + " " + t("dias")}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </section>
    </section>
  );
}
