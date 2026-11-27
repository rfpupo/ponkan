import { Users, Languages, Info, Settings, Heart, Save } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
} from "@/components/ui/dropdown-menu";
import flagBr from "flag-icons/flags/4x3/br.svg";
import flagEs from "flag-icons/flags/4x3/es.svg";
import flagUs from "flag-icons/flags/4x3/us.svg";
import ponkanLogoHd from "../ponkan-logo-hd.webp";
import ponkanLogoLight from "../ponkan-logo-light.png";
import type { Language, Translate } from "../i18n";
import type { Profile } from "../types";

type Props = {
  t: Translate;
  language: Language;
  setLanguage: (value: Language) => void;
  openProfiles: () => void;
  openAbout: () => void;
  openSettings: () => void;
  listReady: boolean;
  initializing: boolean;
  profiles: Profile[];
  favoriteCount: number;
  lastSaved: Profile | undefined;
};
export function HomePage({
  t,
  language,
  setLanguage,
  openProfiles,
  openAbout,
  openSettings,
  listReady,
  initializing,
  profiles,
  favoriteCount,
  lastSaved,
}: Props) {
  return (
    <section className="ponkan-home-hero" aria-label={t("Início")}>
      <img
        className="ponkan-home-logo ponkan-theme-dark"
        src={ponkanLogoHd}
        alt="pon//kan."
        draggable={false}
      />
      <img
        className="ponkan-home-logo ponkan-theme-light"
        src={ponkanLogoLight}
        alt="pon//kan."
        draggable={false}
      />
      <nav className="ponkan-home-nav" aria-label={t("Navegação principal")}>
        <Button size="lg" className="ponkan-home-primary" onClick={openProfiles}>
          <Users size={17} /> {t("Perfis").toLocaleUpperCase(language)}
        </Button>
      </nav>
      <div className="ponkan-home-corner" aria-label={t("Navegação principal")}>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              size="icon-lg"
              variant="ghost"
              className="ponkan-language-shortcut"
              aria-label={t("Idioma")}
              title={t("Idioma")}
            >
              <Languages size={24} />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            align="end"
            side="top"
            sideOffset={9}
            className="ponkan-language-menu"
          >
            <DropdownMenuRadioGroup
              value={language}
              onValueChange={(value) => setLanguage(value as Language)}
            >
              <DropdownMenuRadioItem value="pt-BR">
                <img className="ponkan-flag" src={flagBr} alt="" /> Português
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="es-ES">
                <img className="ponkan-flag" src={flagEs} alt="" /> Español
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="en-US">
                <img className="ponkan-flag" src={flagUs} alt="" /> English
              </DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
        <Button
          size="icon-lg"
          variant="ghost"
          aria-label={t("Sobre")}
          title={t("Sobre")}
          onClick={openAbout}
        >
          <Info size={21} />
        </Button>
        <Button
          size="icon-lg"
          variant="ghost"
          className="ponkan-settings-shortcut"
          aria-label={t("Configurações")}
          title={t("Configurações")}
          onClick={openSettings}
        >
          <Settings size={21} />
        </Button>
      </div>
      {listReady && !initializing && (
        <div className="ponkan-home-meta" aria-label={t("Informações dos perfis")} hidden>
          <button type="button" onClick={openProfiles}>
            <Users size={15} />
            <span>
              {profiles.length === 0
                ? t("Nenhum perfil salvo")
                : profiles.length === 1
                  ? t("1 perfil salvo")
                  : `${profiles.length} ${t("perfis salvos")}`}
            </span>
          </button>
          {favoriteCount > 0 && (
            <button type="button" onClick={openProfiles}>
              <Heart size={15} />
              <span>
                {favoriteCount} {favoriteCount === 1 ? t("favorito") : t("favoritos")}
              </span>
            </button>
          )}
          {lastSaved && (
            <button type="button" onClick={openProfiles} title={lastSaved.name}>
              <Save size={15} />
              <span>
                {t("Último salvo:")} <strong>{lastSaved.name}</strong>
              </span>
            </button>
          )}
        </div>
      )}
    </section>
  );
}
