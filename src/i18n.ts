export type Language = "pt-BR" | "es-ES" | "en-US";
export type Translate = (key: string) => string;
const dictionary: Record<string, { en: string; es: string }> = {
  "Ponkan não é endossado pela Riot Games e não reflete as opiniões da Riot Games ou de qualquer pessoa oficialmente envolvida na produção ou administração das propriedades da Riot Games. Riot Games e todas as propriedades associadas são marcas comerciais ou registradas da Riot Games, Inc.":
    {
      en: "Ponkan isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.",
      es: "Ponkan no está respaldado por Riot Games y no refleja las opiniones de Riot Games ni de ninguna persona oficialmente involucrada en la producción o gestión de las propiedades de Riot Games. Riot Games y todas las propiedades asociadas son marcas comerciales o registradas de Riot Games, Inc.",
    },
  "Existe uma aplicação interrompida. Os dados foram preservados; a recuperação precisa ser verificada.":
    {
      en: "An application was interrupted. Your data has been preserved; recovery must be verified.",
      es: "Se interrumpió la aplicación de un perfil. Tus datos se conservaron; es necesario verificar la recuperación.",
    },
  "A nuvem mudou de forma inesperada. Recuperação automática interrompida.": {
    en: "Cloud settings changed unexpectedly. Automatic recovery stopped.",
    es: "La configuración en la nube cambió de forma inesperada. Se detuvo la recuperación automática.",
  },
  "As configurações mudaram durante a preparação": {
    en: "Settings changed during preparation",
    es: "La configuración cambió durante la preparación",
  },
  "As configurações da nuvem mudaram durante a operação": {
    en: "Cloud settings changed during the operation",
    es: "La configuración en la nube cambió durante la operación",
  },
  "A nuvem não confirmou o perfil aplicado. Backup preservado.": {
    en: "The cloud did not confirm the applied profile. Backup preserved.",
    es: "La nube no confirmó el perfil aplicado. Se conservó la copia de seguridad.",
  },
  "Falha no registro de recuperação": {
    en: "Recovery record failed",
    es: "Error del registro de recuperación",
  },
  "Outra operação está em andamento.": {
    en: "Another operation is in progress.",
    es: "Hay otra operación en curso.",
  },
  "Ir para": { en: "Go to", es: "Ir a" },
  Copiar: { en: "Copy", es: "Copiar" },
  Colar: { en: "Paste", es: "Pegar" },
  Recortar: { en: "Cut", es: "Cortar" },
  "Selecionar tudo": { en: "Select all", es: "Seleccionar todo" },
  "Código de compartilhamento": { en: "Share code", es: "Código para compartir" },
  "Área de transferência vazia.": { en: "Clipboard is empty.", es: "El portapapeles está vacío." },
  "Não foi possível ler a área de transferência.": {
    en: "Could not read the clipboard.",
    es: "No se pudo leer el portapapeles.",
  },
  "Não foi possível copiar o texto.": {
    en: "Could not copy the text.",
    es: "No se pudo copiar el texto.",
  },
  "Texto grande demais para copiar.": {
    en: "Text is too large to copy.",
    es: "Texto demasiado grande para copiar.",
  },
  "Abrir lixeira": { en: "Open recycle bin", es: "Abrir papelera" },
  "Voltar às configurações": { en: "Back to settings", es: "Volver a configuración" },
  Lixeira: { en: "Recycle bin", es: "Papelera" },
  "Voltar à lixeira": { en: "Back to recycle bin", es: "Volver a la papelera" },
  "Restaurar tudo": { en: "Restore all", es: "Restaurar todo" },
  "Esvaziar lixeira": { en: "Empty recycle bin", es: "Vaciar papelera" },
  "Perfis excluídos": { en: "Deleted profiles", es: "Perfiles eliminados" },
  "Excluído em": { en: "Deleted on", es: "Eliminado el" },
  "Excluir permanentemente": { en: "Delete permanently", es: "Eliminar permanentemente" },
  "A lixeira está vazia": { en: "The recycle bin is empty", es: "La papelera está vacía" },
  "Esta ação é permanente e não pode ser desfeita.": {
    en: "This action is permanent and cannot be undone.",
    es: "Esta acción es permanente y no se puede deshacer.",
  },
  "Perfil movido para a lixeira": {
    en: "Profile moved to recycle bin",
    es: "Perfil movido a la papelera",
  },
  "Perfil restaurado": { en: "Profile restored", es: "Perfil restaurado" },
  "Perfis restaurados": { en: "Profiles restored", es: "Perfiles restaurados" },
  "Perfis excluídos permanentemente": {
    en: "Profiles permanently deleted",
    es: "Perfiles eliminados permanentemente",
  },
  "O perfil será movido para a lixeira.": {
    en: "The profile will be moved to the recycle bin.",
    es: "El perfil se moverá a la papelera.",
  },
  "Perfil restaurado, mas não foi possível salvar a imagem.": {
    en: "Profile restored, but the image could not be saved.",
    es: "Perfil restaurado, pero no se pudo guardar la imagen.",
  },
  Comportamento: { en: "Behavior", es: "Comportamiento" },
  "Iniciar com o Windows": { en: "Start with Windows", es: "Iniciar con Windows" },
  "Iniciar minimizado na bandeja": {
    en: "Start minimized to tray",
    es: "Iniciar minimizado en la bandeja",
  },
  "Continuar em segundo plano ao fechar": {
    en: "Keep running in background when closed",
    es: "Seguir en segundo plano al cerrar",
  },
  Armazenamento: { en: "Storage", es: "Almacenamiento" },
  "Limpar lixeira automaticamente": {
    en: "Automatically clean recycle bin",
    es: "Limpiar papelera automáticamente",
  },
  "Prazo contado a partir da exclusão de cada perfil.": {
    en: "Retention starts when each profile is deleted.",
    es: "El plazo comienza al eliminar cada perfil.",
  },
  Nunca: { en: "Never", es: "Nunca" },
  "1 dia": { en: "1 day", es: "1 día" },
  dias: { en: "days", es: "días" },
  "Aguarde a operação atual terminar.": {
    en: "Wait for the current operation to finish.",
    es: "Espera a que termine la operación actual.",
  },
  "Lixeira indisponível.": { en: "Recycle bin unavailable.", es: "Papelera no disponible." },
  "Registro da lixeira inválido.": {
    en: "Invalid recycle bin record.",
    es: "Registro de papelera no válido.",
  },
  "Pasta de lixeira insegura.": {
    en: "Unsafe recycle bin directory.",
    es: "Carpeta de papelera insegura.",
  },
  "Perfil já existente.": { en: "Profile already exists.", es: "El perfil ya existe." },
  "Perfil ainda ativo.": { en: "Profile is still active.", es: "El perfil aún está activo." },
  "Prazo inválido.": { en: "Invalid retention period.", es: "Plazo no válido." },
  "Preferência inválida.": { en: "Invalid preference.", es: "Preferencia no válida." },
  "Preferências indisponíveis.": {
    en: "Preferences unavailable.",
    es: "Preferencias no disponibles.",
  },
  "Preferências de desktop inválidas.": {
    en: "Invalid desktop preferences.",
    es: "Preferencias de escritorio no válidas.",
  },
  "O aplicativo está encerrando.": {
    en: "The app is shutting down.",
    es: "La aplicación se está cerrando.",
  },
  Importar: { en: "Import", es: "Importar" },
  "Copiar código": { en: "Copy code", es: "Copiar código" },
  "Código copiado": { en: "Code copied", es: "Código copiado" },
  "Aguarde a cópia do código.": {
    en: "Wait for the code to be copied.",
    es: "Espera a que se copie el código.",
  },
  "Importar arquivo (.pkn)": { en: "Import file (.pkn)", es: "Importar archivo (.pkn)" },
  "Importar código": { en: "Import code", es: "Importar código" },
  "Código do perfil": { en: "Profile code", es: "Código del perfil" },
  "Código de perfil inválido ou incompatível.": {
    en: "Invalid or incompatible profile code.",
    es: "Código de perfil no válido o incompatible.",
  },
  "Código não encontrado ou indisponível.": {
    en: "Code not found or unavailable.",
    es: "Código no encontrado o no disponible.",
  },
  "Não foi possível conectar ao is.gd. Tente novamente.": {
    en: "Could not connect to is.gd. Try again.",
    es: "No se pudo conectar a is.gd. Inténtalo de nuevo.",
  },
  "Limite do is.gd atingido. Aguarde um minuto.": {
    en: "is.gd rate limit reached. Wait a minute.",
    es: "Límite de is.gd alcanzado. Espera un minuto.",
  },
  "Perfil grande demais para compartilhar por código.": {
    en: "Profile too large to share by code.",
    es: "Perfil demasiado grande para compartir por código.",
  },
  Compartilhar: { en: "Share", es: "Compartir" },
  "Compartilhar perfil": { en: "Share profile", es: "Compartir perfil" },
  "Como deseja compartilhar o perfil?": {
    en: "How would you like to share the profile?",
    es: "¿Cómo quieres compartir el perfil?",
  },
  "Gerar código de compartilhamento": {
    en: "Generate share code",
    es: "Generar código para compartir",
  },
  "Exportar arquivo (.pkn)": { en: "Export file (.pkn)", es: "Exportar archivo (.pkn)" },
  Sim: { en: "Yes", es: "Sí" },
  Não: { en: "No", es: "No" },
  "Voltar aos perfis": { en: "Back to profiles", es: "Volver a perfiles" },
  "Link de perfil inválido ou incompatível.": {
    en: "Invalid or incompatible profile link.",
    es: "Enlace de perfil no válido o incompatible.",
  },
  "Perfil grande demais para compartilhar por link.": {
    en: "Profile too large to share via link.",
    es: "Perfil demasiado grande para compartir por enlace.",
  },
  "Configuração do link não suportada.": {
    en: "Unsupported link setting.",
    es: "Configuración del enlace no compatible.",
  },
  "Valor de configuração inválido.": {
    en: "Invalid setting value.",
    es: "Valor de configuración no válido.",
  },
  "Configuração de vídeo não suportada.": {
    en: "Unsupported video setting.",
    es: "Configuración de vídeo no compatible.",
  },
  "Configuração de vídeo ausente neste computador.": {
    en: "Video setting is missing on this computer.",
    es: "Falta una configuración de vídeo en este equipo.",
  },
  "Não foi possível autenticar no Riot Client. Verifique sua conta e tente novamente.": {
    en: "Could not authenticate with Riot Client. Check your account and try again.",
    es: "No se pudo autenticar en Riot Client. Comprueba tu cuenta e inténtalo de nuevo.",
  },
  "O acesso ao serviço da Riot foi recusado. Verifique sua conta.": {
    en: "Access to the Riot service was denied. Check your account.",
    es: "Se denegó el acceso al servicio de Riot. Comprueba tu cuenta.",
  },
  "Não foi possível obter a sessão da Riot. Verifique se você está conectado.": {
    en: "Could not obtain the Riot session. Check that you are signed in.",
    es: "No se pudo obtener la sesión de Riot. Comprueba que hayas iniciado sesión.",
  },
  "Muitas solicitações à Riot. Aguarde um momento e tente novamente.": {
    en: "Too many requests to Riot. Wait a moment and try again.",
    es: "Demasiadas solicitudes a Riot. Espera un momento e inténtalo de nuevo.",
  },
  "O serviço da Riot está indisponível no momento. Tente novamente.": {
    en: "The Riot service is currently unavailable. Try again.",
    es: "El servicio de Riot no está disponible en este momento. Inténtalo de nuevo.",
  },
  "O serviço da Riot não aceitou a solicitação. Tente novamente.": {
    en: "The Riot service did not accept the request. Try again.",
    es: "El servicio de Riot no aceptó la solicitud. Inténtalo de nuevo.",
  },
  "Não foi possível concluir a comunicação com a Riot. Tente novamente.": {
    en: "Could not complete communication with Riot. Try again.",
    es: "No se pudo completar la comunicación con Riot. Inténtalo de nuevo.",
  },
  "As configurações mudaram durante o salvamento. Tente novamente.": {
    en: "Settings changed while saving. Try again.",
    es: "La configuración cambió durante el guardado. Inténtalo de nuevo.",
  },
  "Não foi possível encontrar as configurações na nuvem Riot. Tente novamente.": {
    en: "Could not find the Riot cloud settings. Try again.",
    es: "No se encontró la configuración en la nube de Riot. Inténtalo de nuevo.",
  },
  "Feche o VALORANT para carregar o perfil.": {
    en: "Close VALORANT to load the profile.",
    es: "Cierra VALORANT para cargar el perfil.",
  },
  "Não foi possível conectar ao Riot Client. Abra o cliente e verifique sua conexão.": {
    en: "Could not connect to Riot Client. Open it and check your connection.",
    es: "No se pudo conectar a Riot Client. Ábrelo y comprueba tu conexión.",
  },
  "A Riot demorou para responder. Tente novamente.": {
    en: "Riot took too long to respond. Try again.",
    es: "Riot tardó demasiado en responder. Inténtalo de nuevo.",
  },
  "Não foi possível acessar os arquivos. Verifique as permissões e tente novamente.": {
    en: "Could not access files. Check permissions and try again.",
    es: "No se pudo acceder a los archivos. Comprueba los permisos e inténtalo de nuevo.",
  },
  "Não foi possível concluir a operação. Tente novamente.": {
    en: "Could not complete the operation. Try again.",
    es: "No se pudo completar la operación. Inténtalo de nuevo.",
  },
  Restaurar: { en: "Restore", es: "Restaurar" },
  Maximizar: { en: "Maximize", es: "Maximizar" },
  Buscar: { en: "Find", es: "Buscar" },
  "O perfil importado aplicará as configurações locais e os campos de nuvem compatíveis. Dados específicos da conta e do computador serão protegidos.":
    {
      en: "The imported profile applies local settings and compatible cloud fields. Account- and device-specific data are protected.",
      es: "El perfil importado aplicará ajustes locales y campos compatibles de la nube. Los datos específicos de la cuenta y del equipo estarán protegidos.",
    },
  "Importar não altera o jogo. O carregamento exige que a conta de destino tenha configurações locais identificáveis.":
    {
      en: "Importing does not change the game. Loading requires identifiable local settings for the destination account.",
      es: "Importar no cambia el juego. La carga requiere configuración local identificable de la cuenta de destino.",
    },
  "Abra o VALORANT nesta conta ao menos uma vez e feche-o antes de carregar o perfil importado": {
    en: "Open VALORANT on this account at least once and close it before loading the imported profile",
    es: "Abre VALORANT en esta cuenta al menos una vez y ciérralo antes de cargar el perfil importado",
  },
  "Não foi possível identificar a conta de origem no perfil": {
    en: "Could not identify the source account in the profile",
    es: "No se pudo identificar la cuenta de origen en el perfil",
  },
  "Identidade Riot indisponível": {
    en: "Riot identity unavailable",
    es: "Identidad Riot no disponible",
  },
  "Exportar (.pkn)": { en: "Export (.pkn)", es: "Exportar (.pkn)" },
  "Importar perfil": { en: "Import profile", es: "Importar perfil" },
  "Exportar perfil": { en: "Export profile", es: "Exportar perfil" },
  "Perfil importado": { en: "Profile imported", es: "Perfil importado" },
  "Perfil exportado": { en: "Profile exported", es: "Perfil exportado" },
  "Autor (opcional)": { en: "Author (optional)", es: "Autor (opcional)" },
  "Descrição (opcional)": { en: "Description (optional)", es: "Descripción (opcional)" },
  "Configurações portáteis aplicadas": {
    en: "Portable settings applied",
    es: "Configuración portátil aplicada",
  },
  "Perfil importado, mas não foi possível salvar a imagem.": {
    en: "Profile imported, but the image could not be saved.",
    es: "Perfil importado, pero no se pudo guardar la imagen.",
  },
  "Já existe um perfil semelhante. Será criada uma cópia independente.": {
    en: "A similar profile already exists. An independent copy will be created.",
    es: "Ya existe un perfil similar. Se creará una copia independiente.",
  },
  "Edições manuais detectadas e validadas.": {
    en: "Manual edits detected and validated.",
    es: "Ediciones manuales detectadas y validadas.",
  },
  "O arquivo inclui dados originais de configuração e pode conter identificadores da conta e do computador. Compartilhe somente com pessoas de confiança.":
    {
      en: "The file contains original settings and may contain account and computer identifiers. Share only with people you trust.",
      es: "El archivo incluye datos originales y puede contener identificadores de la cuenta y del equipo. Compártelo solo con personas de confianza.",
    },
  "Formato .pkn incompatível": { en: "Incompatible .pkn format", es: "Formato .pkn incompatible" },
  "Arquivo .pkn inválido": { en: "Invalid .pkn file", es: "Archivo .pkn no válido" },
  "Arquivo .pkn muito grande": { en: ".pkn file too large", es: "Archivo .pkn demasiado grande" },
  "Arquivo modificado ou corrompido": {
    en: "Modified or corrupted file",
    es: "Archivo modificado o dañado",
  },
  "Manifesto de integridade inválido": {
    en: "Invalid integrity manifest",
    es: "Manifiesto de integridad no válido",
  },
  "settings.json contém campos alterados sem mapeamento validado": {
    en: "settings.json contains changes without a validated mapping",
    es: "settings.json contiene cambios sin mapeo validado",
  },
  Início: {
    en: "Home",
    es: "Inicio",
  },
  Perfis: {
    en: "Profiles",
    es: "Perfiles",
  },
  PERFIS: {
    en: "PROFILES",
    es: "PERFILES",
  },
  Configurações: {
    en: "Settings",
    es: "Configuración",
  },
  Sobre: {
    en: "About",
    es: "Acerca de",
  },
  "Voltar ao início": {
    en: "Back to home",
    es: "Volver al inicio",
  },
  "Navegação principal": {
    en: "Main navigation",
    es: "Navegación principal",
  },
  "Informações dos perfis": {
    en: "Profile information",
    es: "Información de perfiles",
  },
  "Nenhum perfil salvo": {
    en: "No saved profiles",
    es: "Ningún perfil guardado",
  },
  "1 perfil salvo": {
    en: "1 saved profile",
    es: "1 perfil guardado",
  },
  "perfis salvos": {
    en: "saved profiles",
    es: "perfiles guardados",
  },
  favorito: {
    en: "favorite",
    es: "favorito",
  },
  favoritos: {
    en: "favorites",
    es: "favoritos",
  },
  "Último salvo:": {
    en: "Last saved:",
    es: "Último guardado:",
  },
  "Nenhum perfil por aqui...": {
    en: "No profiles here yet...",
    es: "Todavía no hay perfiles...",
  },
  "Novo perfil": {
    en: "New profile",
    es: "Nuevo perfil",
  },
  "A importação .pkn ainda não está disponível": {
    en: ".pkn import is not available yet",
    es: "La importación .pkn aún no está disponible",
  },
  "Importar (.pkn)": {
    en: "Import (.pkn)",
    es: "Importar (.pkn)",
  },
  "Buscar perfil": {
    en: "Search profiles",
    es: "Buscar perfiles",
  },
  "Buscar perfil...": {
    en: "Search profiles...",
    es: "Buscar perfiles...",
  },
  "Ordenar perfis": {
    en: "Sort profiles",
    es: "Ordenar perfiles",
  },
  "Nome (A – Z)": {
    en: "Name (A – Z)",
    es: "Nombre (A – Z)",
  },
  "Nome (Z – A)": {
    en: "Name (Z – A)",
    es: "Nombre (Z – A)",
  },
  "Mais recentes": {
    en: "Most recent",
    es: "Más recientes",
  },
  "Perfis salvos": {
    en: "Saved profiles",
    es: "Perfiles guardados",
  },
  "Nenhum perfil encontrado.": {
    en: "No profiles found.",
    es: "No se encontraron perfiles.",
  },
  "Perfil atual confirmado": {
    en: "Current profile confirmed",
    es: "Perfil actual confirmado",
  },
  "Configurações atuais correspondem ao perfil": {
    en: "Current settings match this profile",
    es: "La configuración actual coincide con el perfil",
  },
  Aplicar: { en: "Apply", es: "Aplicar" },
  Carregar: {
    en: "Load",
    es: "Cargar",
  },
  "Salvar alterações": {
    en: "Save changes",
    es: "Guardar cambios",
  },
  "Salvar alterações em": {
    en: "Save changes to",
    es: "Guardar cambios en",
  },
  "Configurações correspondentes": {
    en: "Matching settings",
    es: "Configuración coincidente",
  },
  Desfavoritar: {
    en: "Unfavorite",
    es: "Quitar de favoritos",
  },
  "Perfil favoritado": { en: "Profile favorited", es: "Perfil añadido a favoritos" },
  Favoritar: {
    en: "Favorite",
    es: "Añadir a favoritos",
  },
  "Remover dos favoritos": {
    en: "Remove from favorites",
    es: "Quitar de favoritos",
  },
  "Opções de": {
    en: "Options for",
    es: "Opciones de",
  },
  Renomear: {
    en: "Rename",
    es: "Renombrar",
  },
  "Alterar imagem": {
    en: "Change image",
    es: "Cambiar imagen",
  },
  "Remover imagem": {
    en: "Remove image",
    es: "Eliminar imagen",
  },
  Excluir: {
    en: "Delete",
    es: "Eliminar",
  },
  Aparência: {
    en: "Appearance",
    es: "Apariencia",
  },
  Escuro: {
    en: "Dark",
    es: "Oscuro",
  },
  Claro: {
    en: "Light",
    es: "Claro",
  },
  Sistema: {
    en: "System",
    es: "Sistema",
  },
  Idioma: {
    en: "Language",
    es: "Idioma",
  },
  "Confirmar antes de carregar": {
    en: "Confirm before loading",
    es: "Confirmar antes de cargar",
  },
  "Confirmar antes de sobrescrever": {
    en: "Confirm before overwriting",
    es: "Confirmar antes de sobrescribir",
  },
  "Backup automático": {
    en: "Automatic backup",
    es: "Copia de seguridad automática",
  },
  "Cria uma cópia das configurações atuais antes de carregar outro perfil.": {
    en: "Keeps a copy of current settings before loading another profile.",
    es: "Guarda una copia de la configuración actual antes de cargar otro perfil.",
  },
  Geral: {
    en: "General",
    es: "General",
  },
  Segurança: {
    en: "Safety",
    es: "Seguridad",
  },
  Sobrescrever: {
    en: "Overwrite",
    es: "Sobrescribir",
  },
  "As configurações atuais substituirão o perfil salvo.": {
    en: "Current settings will replace the saved profile.",
    es: "La configuración actual sustituirá al perfil guardado.",
  },
  Cancelar: {
    en: "Cancel",
    es: "Cancelar",
  },
  Salvar: {
    en: "Save",
    es: "Guardar",
  },
  "Renomear perfil": {
    en: "Rename profile",
    es: "Renombrar perfil",
  },
  "Salvar perfil": {
    en: "Save profile",
    es: "Guardar perfil",
  },
  "Nome do perfil": {
    en: "Profile name",
    es: "Nombre del perfil",
  },
  "As configurações atuais serão substituídas. Um backup será criado automaticamente.": {
    en: "Current settings will be replaced. A backup will be created automatically.",
    es: "La configuración actual se sustituirá. Se creará una copia de seguridad automáticamente.",
  },
  "As configurações atuais serão substituídas sem criar um backup permanente.": {
    en: "Current settings will be replaced without creating a permanent backup.",
    es: "La configuración actual se sustituirá sin crear una copia de seguridad permanente.",
  },
  "O perfil deixará de aparecer na lista.": {
    en: "The profile will no longer appear in the list.",
    es: "El perfil dejará de aparecer en la lista.",
  },
  Versão: {
    en: "Version",
    es: "Versión",
  },
  "Gerenciador de perfis do VALORANT.": {
    en: "VALORANT profile manager.",
    es: "Gestor de perfiles de VALORANT.",
  },
  "Nome atualizado": {
    en: "Name updated",
    es: "Nombre actualizado",
  },
  "Perfil salvo": {
    en: "Profile saved",
    es: "Perfil guardado",
  },
  "Perfil carregado": {
    en: "Profile loaded",
    es: "Perfil cargado",
  },
  "Perfil atualizado": {
    en: "Profile updated",
    es: "Perfil actualizado",
  },
  "Perfil excluído": {
    en: "Profile deleted",
    es: "Perfil eliminado",
  },
  Minimizar: {
    en: "Minimize",
    es: "Minimizar",
  },
  Fechar: {
    en: "Close",
    es: "Cerrar",
  },
  "Escolha uma imagem PNG, JPEG ou WebP de até 10 MB.": {
    en: "Choose a PNG, JPEG or WebP image up to 10 MB.",
    es: "Elige una imagen PNG, JPEG o WebP de hasta 10 MB.",
  },
  "Não foi possível preparar a imagem.": {
    en: "Could not prepare the image.",
    es: "No se pudo preparar la imagen.",
  },
  "Falha ao acessar arquivos": {
    en: "Could not access files",
    es: "No se pudo acceder a los archivos",
  },
  "AppData não encontrado": {
    en: "AppData not found",
    es: "AppData no encontrado",
  },
  "Nome de perfil inválido": {
    en: "Invalid profile name",
    es: "Nombre de perfil no válido",
  },
  "Feche o VALORANT antes de continuar": {
    en: "Close VALORANT before continuing",
    es: "Cierra VALORANT antes de continuar",
  },
  "Link simbólico em configurações": {
    en: "Symbolic link in settings",
    es: "Enlace simbólico en la configuración",
  },
  "Configurações locais não encontradas": {
    en: "Local settings not found",
    es: "Configuración local no encontrada",
  },
  "Pasta de destino já existe": {
    en: "Destination folder already exists",
    es: "La carpeta de destino ya existe",
  },
  "Arquivos restaurados diferem do perfil": {
    en: "Restored files differ from the profile",
    es: "Los archivos restaurados difieren del perfil",
  },
  "Lista de perfis danificada": {
    en: "Corrupted profile list",
    es: "Lista de perfiles dañada",
  },
  "Caminho inválido": {
    en: "Invalid path",
    es: "Ruta no válida",
  },
  "Identificador inválido": {
    en: "Invalid identifier",
    es: "Identificador no válido",
  },
  "Perfil não encontrado": {
    en: "Profile not found",
    es: "Perfil no encontrado",
  },
  "Perfil da nuvem vazio": {
    en: "Empty cloud profile",
    es: "Perfil de la nube vacío",
  },
  "Arquivos locais ausentes": {
    en: "Local files missing",
    es: "Faltan archivos locales",
  },
  "Configurações da nuvem vazias": {
    en: "Empty cloud settings",
    es: "Configuración de la nube vacía",
  },
  "Já existe um perfil com esse nome": {
    en: "A profile with this name already exists",
    es: "Ya existe un perfil con ese nombre",
  },
  "Nome já utilizado": {
    en: "Name already in use",
    es: "Nombre ya utilizado",
  },
  "Pasta de configurações do VALORANT não encontrada": {
    en: "VALORANT settings folder not found",
    es: "Carpeta de configuración de VALORANT no encontrada",
  },
  "Pasta de configuração inválida": {
    en: "Invalid settings folder",
    es: "Carpeta de configuración no válida",
  },
  "Abra e entre no Riot Client": {
    en: "Open and sign in to Riot Client",
    es: "Abre e inicia sesión en Riot Client",
  },
  "Sessão Riot inválida": {
    en: "Invalid Riot session",
    es: "Sesión de Riot no válida",
  },
  "Porta Riot inválida": {
    en: "Invalid Riot port",
    es: "Puerto de Riot no válido",
  },
  "Falha ao conectar ao Riot Client": {
    en: "Could not connect to Riot Client",
    es: "No se pudo conectar a Riot Client",
  },
  "Riot Client indisponível": {
    en: "Riot Client unavailable",
    es: "Riot Client no disponible",
  },
  "Riot Client sem sessão válida": {
    en: "Riot Client has no valid session",
    es: "Riot Client no tiene una sesión válida",
  },
  "Token de acesso não disponível": {
    en: "Access token unavailable",
    es: "Token de acceso no disponible",
  },
  "Token Riot não disponível": {
    en: "Riot token unavailable",
    es: "Token de Riot no disponible",
  },
  "Falha na comunicação com a Riot": {
    en: "Riot communication failed",
    es: "Error de comunicación con Riot",
  },
  "Não foi possível consultar a nuvem Riot": {
    en: "Could not read Riot cloud settings",
    es: "No se pudo consultar la nube de Riot",
  },
  "A Riot recusou a consulta": {
    en: "Riot rejected the request",
    es: "Riot rechazó la consulta",
  },
  "Resposta da Riot inválida": {
    en: "Invalid Riot response",
    es: "Respuesta de Riot no válida",
  },
  "Nuvem Riot retornou configurações vazias": {
    en: "Riot cloud returned empty settings",
    es: "La nube de Riot devolvió una configuración vacía",
  },
  "Backup da nuvem vazio": {
    en: "Empty cloud backup",
    es: "Copia de seguridad de la nube vacía",
  },
  "Não foi possível salvar na nuvem Riot": {
    en: "Could not save to Riot cloud",
    es: "No se pudo guardar en la nube de Riot",
  },
  "A Riot recusou a restauração": {
    en: "Riot rejected the restore",
    es: "Riot rechazó la restauración",
  },
  "A nuvem não confirmou o perfil restaurado": {
    en: "Cloud did not confirm restored profile",
    es: "La nube no confirmó el perfil restaurado",
  },
  "Recuperação da nuvem falhou": {
    en: "Cloud recovery failed",
    es: "Error de recuperación de la nube",
  },
};
export function readLanguage(): Language {
  const value = localStorage.getItem("ponkan-language");
  return value === "en-US" || value === "es-ES" ? value : "pt-BR";
}
export function translate(language: Language, key: string): string {
  return language === "pt-BR"
    ? key
    : (dictionary[key]?.[language === "en-US" ? "en" : "es"] ?? key);
}
export function translateError(language: Language, error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  if (language === "pt-BR") return message;
  const keys = Object.keys(dictionary);
  const exact = dictionary[message];
  if (exact) return translate(language, message);
  const rollback = ". Recuperação da nuvem falhou: ";
  if (message.includes(rollback)) {
    const [first, ...rest] = message.split(rollback);
    return `${translateError(language, first)}. ${translate(language, "Recuperação da nuvem falhou")}: ${translateError(language, rest.join(rollback))}`;
  }
  const prefix = keys
    .sort((a, b) => b.length - a.length)
    .find((key) => message.startsWith(key + ":") || message.startsWith(key + " ("));
  return prefix ? translate(language, prefix) + message.slice(prefix.length) : message;
}

/** Friendly notifications; keep technical HTTP status separate from user-facing text. */
export function notificationError(language: Language, error: unknown): string {
  const raw = error instanceof Error ? error.message : String(error);
  const rollback = ". Recuperação da nuvem falhou: ";
  if (raw.includes(rollback)) {
    const [first, ...rest] = raw.split(rollback);
    return (
      notificationError(language, first) +
      ". " +
      translate(language, "Recuperação da nuvem falhou") +
      ": " +
      notificationError(language, rest.join(rollback))
    );
  }
  const http = raw.match(/ \(HTTP (\d{3})\)$/);
  if (http) {
    console.warn("[ponkan] Riot HTTP", Number(http[1]));
    const text = raw.slice(0, http.index);
    if (dictionary[text]) return translate(language, text);
  }
  if (raw === "Feche o VALORANT antes de continuar")
    return translate(language, "Feche o VALORANT para carregar o perfil.");
  if (raw.startsWith("Falha ao acessar arquivos:"))
    return translate(
      language,
      "Não foi possível acessar os arquivos. Verifique as permissões e tente novamente.",
    );
  if (raw === "Riot Client indisponível")
    return translate(
      language,
      "Não foi possível conectar ao Riot Client. Abra o cliente e verifique sua conexão.",
    );
  if (
    raw === "Sessão Riot inválida" ||
    raw === "Token de acesso não disponível" ||
    raw === "Token Riot não disponível"
  )
    return translate(
      language,
      "Não foi possível obter a sessão da Riot. Verifique se você está conectado.",
    );
  // Retain existing specific validation messages, including portable profile details.
  if (raw === "[object Object]" || raw === "undefined" || raw === "null" || !raw.trim())
    return translate(language, "Não foi possível concluir a operação. Tente novamente.");
  return translateError(language, raw);
}
