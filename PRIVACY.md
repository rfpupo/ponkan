# Privacy notice

Ponkan is free and open-source, with no developer-operated advertising or analytics. This notice describes the 0.3.0 beta candidate; native validation remains incomplete.

- Settings profiles, history, recycle bin, avatars and preferences are stored locally. Full snapshots and exports can include account identifiers and device-specific settings. Do not publish them or attach them to reports.
- Ponkan reads the local Riot Client lockfile and retrieves the session authorization information to communicate with Riot's player preference service. The tokens are used in memory by the current backend. The local HTTPS client accepts the local Riot certificate; cloud communication uses TLS.
- Saving reads game preferences; applying can write them and change the local VALORANT Config tree. Application is blocked while the game is running.
- Generating a short share code sends a portable compressed profile, including name, optional author/description and allowlisted settings, to is.gd. That service may receive network metadata and retain links. Codes are not confidential and are not encrypted. Importing a short code also contacts is.gd. Creating a code is optional; file export remains local.
- Recovery records and optional backups can contain personal settings and account identifiers. They are stored locally and must never be submitted to GitHub or external services.
- Ponkan does not operate a developer-hosted account or telemetry backend in the current architecture. Riot and is.gd have their own privacy practices.
- Users can delete stored profiles through the application; recycle bin expiry is configurable. Backup retention and migration remain under review.
- There is no automatic updater or developer-hosted service. Do not submit sensitive data in public issues.

Riot privacy notice: https://www.riotgames.com/en/privacy-notice
is.gd privacy policy: https://is.gd/privacy.php

Portuguese summary: Perfis, históricos, avatares e preferências ficam no computador. Salvar consulta a Riot; aplicar modifica os dados locais e da nuvem. Gerar código envia os dados portáteis ao is.gd; não é um compartilhamento privado. Arquivos completos, backups e registros de recuperação podem conter dados pessoais. Não envie esses arquivos a terceiros.
