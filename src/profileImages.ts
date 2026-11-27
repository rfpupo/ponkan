const PREFIX = "ponkan-avatar:";
export function loadProfileImage(id: string): string | undefined {
  try {
    const value = localStorage.getItem(PREFIX + id);
    return value?.startsWith("data:image/jpeg;base64,") ? value : undefined;
  } catch {
    return undefined;
  }
}
export function removeProfileImage(id: string) {
  localStorage.removeItem(PREFIX + id);
}
export async function saveProfileImage(id: string, file: File): Promise<string> {
  if (
    !["image/png", "image/jpeg", "image/webp"].includes(file.type) ||
    file.size > 10 * 1024 * 1024
  ) {
    throw new Error("Escolha uma imagem PNG, JPEG ou WebP de até 10 MB.");
  }
  const bitmap = await createImageBitmap(file);
  try {
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 128;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Não foi possível preparar a imagem.");
    const side = Math.min(bitmap.width, bitmap.height);
    ctx.fillStyle = "#0c0c0c";
    ctx.fillRect(0, 0, 128, 128);
    ctx.drawImage(
      bitmap,
      (bitmap.width - side) / 2,
      (bitmap.height - side) / 2,
      side,
      side,
      0,
      0,
      128,
      128,
    );
    const value = canvas.toDataURL("image/jpeg", 0.86);
    localStorage.setItem(PREFIX + id, value);
    return value;
  } finally {
    bitmap.close();
  }
}
