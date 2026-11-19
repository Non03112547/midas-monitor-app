import { check } from '@tauri-apps/plugin-updater';

let isCheckingUpdate = false;
let hasCheckedStartup = false;

export interface UpdateCheckOptions {
  isStartup?: boolean;
}

/**
 * Checks for an application update and prompts the user via a native OS dialog
 * before downloading and installing. Prevents concurrent or duplicate checks.
 */
export async function checkAndPromptForUpdate(
  options: UpdateCheckOptions = {}
): Promise<void> {
  const { isStartup = false } = options;

  if (isCheckingUpdate) {
    return;
  }

  if (isStartup && hasCheckedStartup) {
    return;
  }

  isCheckingUpdate = true;
  if (isStartup) {
    hasCheckedStartup = true;
  }

  try {
    const update = await check();
    if (!update) {
      return;
    }

    const notes = update.body?.trim();
    const notesSection = notes ? `\n\nรายละเอียดการอัปเดต:\n${notes}` : '';
    const confirmMessage =
      `พบ MD Monitor เวอร์ชันใหม่ v${update.version} (เวอร์ชันปัจจุบัน v${update.currentVersion})` +
      `${notesSection}\n\nต้องการดาวน์โหลดและติดตั้งอัปเดตเลยหรือไม่?`;

    // Use native browser/WebView2 confirm dialog because the OSD main window uses set_ignore_cursor(true).
    const confirmed = window.confirm(confirmMessage);

    if (!confirmed) {
      console.info(`[Updater] User cancelled update to v${update.version}.`);
      await update.close().catch(() => {});
      return;
    }

    console.info(`[Updater] User confirmed update to v${update.version}. Downloading and installing...`);

    try {
      await update.downloadAndInstall();
      window.alert(
        `ติดตั้ง MD Monitor เวอร์ชัน v${update.version} เรียบร้อยแล้ว\n\nกรุณารีสตาร์ทแอปพลิเคชันหากระบบไม่ได้เปิดใหม่อัตโนมัติ`
      );
    } catch (installErr) {
      console.error('[Updater] Update download/install failed:', installErr);
      await update.close().catch(() => {});
      window.alert(
        `ไม่สามารถดาวน์โหลดหรือติดตั้งอัปเดต v${update.version} ได้\nกรุณาลองใหม่อีกครั้งในภายหลัง`
      );
    }
  } catch (err) {
    console.warn('[Updater] Update check skipped or failed:', err);
  } finally {
    isCheckingUpdate = false;
  }
}
