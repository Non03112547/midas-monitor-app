import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

interface AppConfig {
  pos_x: number;
  pos_y: number;
  bg_r: number;
  bg_g: number;
  bg_b: number;
  bg_alpha: number;
  font_size: number;
  text_color: string;
  outline_color: string;
  has_outline: boolean;
  show_cpu: boolean;
  show_gpu: boolean;
  show_ram: boolean;
  show_disk: boolean;
  show_net: boolean;
  show_ping: boolean;
  show_fps: boolean;
  layout_direction: string;
  autostart: boolean;
  language: string;
}

// Translations matching Python monitor.pyw
const i18n: Record<string, Record<string, string>> = {
  th: {
    configTitle: "⚙️ ตั้งค่า (Config)",
    langBtn: "Language: TH",
    btnAdjustPos: "📐 ตั้งค่าตำแหน่ง (Adjust Position)",
    radarRes: "จอหลัก: ",
    presetTl: "↖ บนซ้าย",
    presetTr: "↗ บนขวา",
    presetBl: "↙ ล่างซ้าย",
    presetBr: "↘ ล่างขวา",
    secColors: "🎨 สี (Colors)",
    lblColorBg: "เลือกสีพื้นหลัง (BG Color)",
    lblColorText: "เลือกสีข้อความ (Text Color)",
    lblColorOutline: "เลือกสีขอบข้อความ (Outline Color)",
    secSliders: "🎚️ ความทึบและขนาด (Sliders)",
    lblOpacity: "ความทึบพื้นหลัง:",
    lblFontSize: "ขนาดตัวอักษร:",
    secLayout: "🔲 การจัดวาง (Layout & Outline)",
    lblChkOutline: "เปิดใช้งานขอบข้อความ (Text Outline)",
    lblLayoutDir: "รูปแบบการจัดวาง (Layout):",
    layoutVertical: "Vertical (แนวตั้ง)",
    layoutHorizontal: "Horizontal (แนวนอน)",
    secMonitors: "📊 การแสดงผลฮาร์ดแวร์ (Display Items)",
    lblChkCpu: "แสดง CPU",
    lblChkGpu: "แสดง GPU",
    lblChkRam: "แสดง RAM",
    lblChkDisk: "แสดง DISK (ทุกไดรฟ์)",
    lblChkNet: "แสดง Network",
    lblChkPing: "แสดง Ping (Latency)",
    lblChkFps: "แสดง FPS",
    secSystem: "🚀 ระบบ (System)",
    lblChkAutostart: "เปิดโปรแกรมตอนเปิดเครื่อง (Auto-Start)",
    btnReset: "คืนค่าเริ่มต้น (Reset to Defaults)",
    btnDone: "เสร็จสิ้น (Done)",
  },
  en: {
    configTitle: "⚙️ Settings (Config)",
    langBtn: "Language: EN",
    btnAdjustPos: "📐 Adjust Position",
    radarRes: "Primary Screen: ",
    presetTl: "↖ Top-Left",
    presetTr: "↗ Top-Right",
    presetBl: "↙ Bottom-Left",
    presetBr: "↘ Bottom-Right",
    secColors: "🎨 Colors",
    lblColorBg: "Choose Background Color",
    lblColorText: "Choose Text Color",
    lblColorOutline: "Choose Outline Color",
    secSliders: "🎚️ Opacity & Font Size",
    lblOpacity: "Background Opacity:",
    lblFontSize: "Font Size:",
    secLayout: "🔲 Layout & Outline",
    lblChkOutline: "Enable Text Outline",
    lblLayoutDir: "Layout Direction:",
    layoutVertical: "Vertical",
    layoutHorizontal: "Horizontal",
    secMonitors: "📊 Display Items",
    lblChkCpu: "Show CPU",
    lblChkGpu: "Show GPU",
    lblChkRam: "Show RAM",
    lblChkDisk: "Show DISK (All Drives)",
    lblChkNet: "Show Network",
    lblChkPing: "Show Ping (Latency)",
    lblChkFps: "Show FPS",
    secSystem: "🚀 System",
    lblChkAutostart: "Run on Startup (Auto-Start)",
    btnReset: "Reset to Defaults",
    btnDone: "Done",
  },
};

// DOM Elements
const configTitle = document.getElementById('config-title')!;
const btnLangToggle = document.getElementById('btn-lang-toggle')!;
const btnToggleRadar = document.getElementById('btn-toggle-radar')!;
const radarSection = document.getElementById('radar-section')!;
const radarRes = document.getElementById('radar-res')!;
const radarCoords = document.getElementById('radar-coords')!;
const screenMap = document.getElementById('screen-map')!;
const appRep = document.getElementById('app-rep')!;

const presetTl = document.getElementById('preset-tl')!;
const presetTr = document.getElementById('preset-tr')!;
const presetBl = document.getElementById('preset-bl')!;
const presetBr = document.getElementById('preset-br')!;

const secColors = document.getElementById('sec-colors')!;
const lblColorBg = document.getElementById('lbl-color-bg')!;
const colorBg = document.getElementById('color-bg') as HTMLInputElement;
const hexBg = document.getElementById('hex-bg')!;

const lblColorText = document.getElementById('lbl-color-text')!;
const colorText = document.getElementById('color-text') as HTMLInputElement;
const hexText = document.getElementById('hex-text')!;

const lblColorOutline = document.getElementById('lbl-color-outline')!;
const colorOutline = document.getElementById('color-outline') as HTMLInputElement;
const hexOutline = document.getElementById('hex-outline')!;

const secSliders = document.getElementById('sec-sliders')!;
const lblOpacity = document.getElementById('lbl-opacity')!;
const badgeOpacity = document.getElementById('badge-opacity')!;
const sliderOpacity = document.getElementById('slider-opacity') as HTMLInputElement;

const lblFontSize = document.getElementById('lbl-font-size')!;
const badgeFontSize = document.getElementById('badge-font-size')!;
const sliderFontSize = document.getElementById('slider-font-size') as HTMLInputElement;

const secLayout = document.getElementById('sec-layout')!;
const chkOutline = document.getElementById('chk-outline') as HTMLInputElement;
const lblChkOutline = document.getElementById('lbl-chk-outline')!;
const lblLayoutDir = document.getElementById('lbl-layout-dir')!;
const selectLayout = document.getElementById('select-layout') as HTMLSelectElement;

const secMonitors = document.getElementById('sec-monitors')!;
const chkCpu = document.getElementById('chk-cpu') as HTMLInputElement;
const lblChkCpu = document.getElementById('lbl-chk-cpu')!;
const chkGpu = document.getElementById('chk-gpu') as HTMLInputElement;
const lblChkGpu = document.getElementById('lbl-chk-gpu')!;
const chkRam = document.getElementById('chk-ram') as HTMLInputElement;
const lblChkRam = document.getElementById('lbl-chk-ram')!;
const chkDisk = document.getElementById('chk-disk') as HTMLInputElement;
const lblChkDisk = document.getElementById('lbl-chk-disk')!;
const chkNet = document.getElementById('chk-net') as HTMLInputElement;
const lblChkNet = document.getElementById('lbl-chk-net')!;
const chkPing = document.getElementById('chk-ping') as HTMLInputElement;
const lblChkPing = document.getElementById('lbl-chk-ping')!;
const chkFps = document.getElementById('chk-fps') as HTMLInputElement;
const lblChkFps = document.getElementById('lbl-chk-fps')!;

const secSystem = document.getElementById('sec-system')!;
const chkAutostart = document.getElementById('chk-autostart') as HTMLInputElement;
const lblChkAutostart = document.getElementById('lbl-chk-autostart')!;

const btnReset = document.getElementById('btn-reset')!;
const btnDone = document.getElementById('btn-done')!;

// App State
let config: AppConfig = {
  pos_x: 28,
  pos_y: 19,
  bg_r: 0,
  bg_g: 0,
  bg_b: 0,
  bg_alpha: 60,
  font_size: 16,
  text_color: "#ff0000",
  outline_color: "#00007f",
  has_outline: true,
  show_cpu: true,
  show_gpu: true,
  show_ram: true,
  show_disk: false,
  show_net: false,
  show_ping: true,
  show_fps: false,
  layout_direction: "vertical",
  autostart: true,
  language: "th"
};

let screenWidth = 1920;
let screenHeight = 1080;
let mainWidth = 280;
let mainHeight = 350;

const mapWidth = 400;
let mapHeight = 225;
let repWidth = 40;
let repHeight = 50;

let isDragging = false;
let dragOffsetX = 0;
let dragOffsetY = 0;

// Color Helpers
function rgbToHex(r: number, g: number, b: number): string {
  return '#' + [r, g, b].map(x => Math.max(0, Math.min(255, x)).toString(16).padStart(2, '0')).join('');
}

function hexToRgb(hex: string): { r: number; g: number; b: number } {
  let c = hex.replace('#', '');
  if (c.length === 3) {
    c = c.split('').map(x => x + x).join('');
  }
  const num = parseInt(c, 16);
  return {
    r: (num >> 16) & 255,
    g: (num >> 8) & 255,
    b: num & 255,
  };
}

function applyLanguage(lang: string) {
  const dict = i18n[lang] || i18n.th;
  configTitle.textContent = dict.configTitle;
  btnLangToggle.textContent = dict.langBtn;
  btnToggleRadar.textContent = dict.btnAdjustPos;
  presetTl.textContent = dict.presetTl;
  presetTr.textContent = dict.presetTr;
  presetBl.textContent = dict.presetBl;
  presetBr.textContent = dict.presetBr;

  secColors.textContent = dict.secColors;
  lblColorBg.textContent = dict.lblColorBg;
  lblColorText.textContent = dict.lblColorText;
  lblColorOutline.textContent = dict.lblColorOutline;

  secSliders.textContent = dict.secSliders;
  lblOpacity.textContent = `${dict.lblOpacity} (${config.bg_alpha}):`;
  lblFontSize.textContent = `${dict.lblFontSize} (${config.font_size}):`;

  secLayout.textContent = dict.secLayout;
  lblChkOutline.textContent = dict.lblChkOutline;
  lblLayoutDir.textContent = dict.lblLayoutDir;
  selectLayout.options[0].textContent = dict.layoutVertical;
  selectLayout.options[1].textContent = dict.layoutHorizontal;

  secMonitors.textContent = dict.secMonitors;
  lblChkCpu.textContent = dict.lblChkCpu;
  lblChkGpu.textContent = dict.lblChkGpu;
  lblChkRam.textContent = dict.lblChkRam;
  lblChkDisk.textContent = dict.lblChkDisk;
  lblChkNet.textContent = dict.lblChkNet;
  lblChkPing.textContent = dict.lblChkPing;
  lblChkFps.textContent = dict.lblChkFps;

  secSystem.textContent = dict.secSystem;
  lblChkAutostart.textContent = dict.lblChkAutostart;

  btnReset.textContent = dict.btnReset;
  btnDone.textContent = dict.btnDone;

  radarRes.textContent = `${dict.radarRes}${screenWidth} x ${screenHeight}`;
}

function populateForm() {
  const bgHex = rgbToHex(config.bg_r, config.bg_g, config.bg_b);
  colorBg.value = bgHex;
  hexBg.textContent = bgHex;

  colorText.value = config.text_color;
  hexText.textContent = config.text_color;

  colorOutline.value = config.outline_color;
  hexOutline.textContent = config.outline_color;

  sliderOpacity.value = config.bg_alpha.toString();
  badgeOpacity.textContent = config.bg_alpha.toString();

  sliderFontSize.value = config.font_size.toString();
  badgeFontSize.textContent = `${config.font_size}px`;

  chkOutline.checked = config.has_outline;
  selectLayout.value = config.layout_direction;

  chkCpu.checked = config.show_cpu;
  chkGpu.checked = config.show_gpu;
  chkRam.checked = config.show_ram;
  chkDisk.checked = config.show_disk;
  chkNet.checked = config.show_net;
  chkPing.checked = config.show_ping;
  chkFps.checked = config.show_fps;

  chkAutostart.checked = config.autostart;

  applyLanguage(config.language);
  updateRadarDisplay();
}

async function saveCurrentConfig() {
  try {
    await invoke('save_config', { config });
  } catch (err) {
    console.error('Failed to save config:', err);
  }
}

// Mini-Map Radar Logic
async function updateRadarMetrics() {
  try {
    const res = await invoke<[number, number]>('get_screen_resolution');
    screenWidth = res[0] || 1920;
    screenHeight = res[1] || 1080;

    const rect = await invoke<[number, number, number, number]>('get_main_rect');
    mainWidth = rect[2] || 280;
    mainHeight = rect[3] || 350;
  } catch (e) {
    console.warn('Could not query screen/window metrics:', e);
  }

  mapHeight = Math.round(mapWidth * (screenHeight / screenWidth));
  screenMap.style.height = `${mapHeight}px`;

  repWidth = Math.max(12, Math.round((mainWidth / screenWidth) * mapWidth));
  repHeight = Math.max(12, Math.round((mainHeight / screenHeight) * mapHeight));

  appRep.style.width = `${repWidth}px`;
  appRep.style.height = `${repHeight}px`;

  const dict = i18n[config.language] || i18n.th;
  radarRes.textContent = `${dict.radarRes}${screenWidth} x ${screenHeight}`;
}

function updateRadarDisplay() {
  const x = Math.round((config.pos_x / screenWidth) * mapWidth);
  const y = Math.round((config.pos_y / screenHeight) * mapHeight);

  const clampedX = Math.max(0, Math.min(x, mapWidth - repWidth));
  const clampedY = Math.max(0, Math.min(y, mapHeight - repHeight));

  appRep.style.left = `${clampedX}px`;
  appRep.style.top = `${clampedY}px`;

  radarCoords.textContent = `X: ${config.pos_x}, Y: ${config.pos_y}`;
}

async function setMainWindowPos(realX: number, realY: number) {
  config.pos_x = realX;
  config.pos_y = realY;
  radarCoords.textContent = `X: ${realX}, Y: ${realY}`;
  try {
    await invoke('set_main_position', { x: realX, y: realY });
  } catch (err) {
    console.error('set_main_position failed:', err);
  }
}

// Radar Drag Handlers
appRep.addEventListener('mousedown', (e) => {
  isDragging = true;
  appRep.classList.add('dragging');
  const repRect = appRep.getBoundingClientRect();
  dragOffsetX = e.clientX - repRect.left;
  dragOffsetY = e.clientY - repRect.top;
  e.preventDefault();
});

window.addEventListener('mousemove', async (e) => {
  if (!isDragging) return;

  const mapRect = screenMap.getBoundingClientRect();
  let newX = e.clientX - mapRect.left - dragOffsetX;
  let newY = e.clientY - mapRect.top - dragOffsetY;

  newX = Math.max(0, Math.min(newX, mapWidth - repWidth));
  newY = Math.max(0, Math.min(newY, mapHeight - repHeight));

  appRep.style.left = `${newX}px`;
  appRep.style.top = `${newY}px`;

  const realX = Math.round((newX / mapWidth) * screenWidth);
  const realY = Math.round((newY / mapHeight) * screenHeight);

  await setMainWindowPos(realX, realY);
});

window.addEventListener('mouseup', () => {
  if (isDragging) {
    isDragging = false;
    appRep.classList.remove('dragging');
    saveCurrentConfig();
  }
});

// Click anywhere on map to jump
screenMap.addEventListener('click', async (e) => {
  if (e.target === appRep) return;
  const mapRect = screenMap.getBoundingClientRect();
  let newX = e.clientX - mapRect.left - Math.round(repWidth / 2);
  let newY = e.clientY - mapRect.top - Math.round(repHeight / 2);

  newX = Math.max(0, Math.min(newX, mapWidth - repWidth));
  newY = Math.max(0, Math.min(newY, mapHeight - repHeight));

  appRep.style.left = `${newX}px`;
  appRep.style.top = `${newY}px`;

  const realX = Math.round((newX / mapWidth) * screenWidth);
  const realY = Math.round((newY / mapHeight) * screenHeight);

  await setMainWindowPos(realX, realY);
  saveCurrentConfig();
});

// Presets
presetTl.addEventListener('click', async () => {
  await setMainWindowPos(20, 20);
  updateRadarDisplay();
  saveCurrentConfig();
});

presetTr.addEventListener('click', async () => {
  const x = Math.max(0, screenWidth - mainWidth - 20);
  await setMainWindowPos(x, 20);
  updateRadarDisplay();
  saveCurrentConfig();
});

presetBl.addEventListener('click', async () => {
  const y = Math.max(0, screenHeight - mainHeight - 60);
  await setMainWindowPos(20, y);
  updateRadarDisplay();
  saveCurrentConfig();
});

presetBr.addEventListener('click', async () => {
  const x = Math.max(0, screenWidth - mainWidth - 20);
  const y = Math.max(0, screenHeight - mainHeight - 60);
  await setMainWindowPos(x, y);
  updateRadarDisplay();
  saveCurrentConfig();
});

// Toggle Radar collapsible box
btnToggleRadar.addEventListener('click', () => {
  radarSection.classList.toggle('hidden');
});

// Settings Inputs Listeners
btnLangToggle.addEventListener('click', () => {
  config.language = config.language === 'th' ? 'en' : 'th';
  applyLanguage(config.language);
  saveCurrentConfig();
});

colorBg.addEventListener('input', () => {
  const rgb = hexToRgb(colorBg.value);
  config.bg_r = rgb.r;
  config.bg_g = rgb.g;
  config.bg_b = rgb.b;
  hexBg.textContent = colorBg.value;
  saveCurrentConfig();
});

colorText.addEventListener('input', () => {
  config.text_color = colorText.value;
  hexText.textContent = colorText.value;
  saveCurrentConfig();
});

colorOutline.addEventListener('input', () => {
  config.outline_color = colorOutline.value;
  hexOutline.textContent = colorOutline.value;
  saveCurrentConfig();
});

sliderOpacity.addEventListener('input', () => {
  const val = parseInt(sliderOpacity.value, 10);
  config.bg_alpha = val;
  badgeOpacity.textContent = val.toString();
  const dict = i18n[config.language] || i18n.th;
  lblOpacity.textContent = `${dict.lblOpacity} (${val}):`;
  saveCurrentConfig();
});

sliderFontSize.addEventListener('input', () => {
  const val = parseInt(sliderFontSize.value, 10);
  config.font_size = val;
  badgeFontSize.textContent = `${val}px`;
  const dict = i18n[config.language] || i18n.th;
  lblFontSize.textContent = `${dict.lblFontSize} (${val}):`;
  saveCurrentConfig();
});

chkOutline.addEventListener('change', () => {
  config.has_outline = chkOutline.checked;
  saveCurrentConfig();
});

selectLayout.addEventListener('change', () => {
  config.layout_direction = selectLayout.value;
  saveCurrentConfig();
});

chkCpu.addEventListener('change', () => {
  config.show_cpu = chkCpu.checked;
  saveCurrentConfig();
});

chkGpu.addEventListener('change', () => {
  config.show_gpu = chkGpu.checked;
  saveCurrentConfig();
});

chkRam.addEventListener('change', () => {
  config.show_ram = chkRam.checked;
  saveCurrentConfig();
});

chkDisk.addEventListener('change', () => {
  config.show_disk = chkDisk.checked;
  saveCurrentConfig();
});

chkNet.addEventListener('change', () => {
  config.show_net = chkNet.checked;
  saveCurrentConfig();
});

chkPing.addEventListener('change', () => {
  config.show_ping = chkPing.checked;
  saveCurrentConfig();
});

chkFps.addEventListener('change', () => {
  config.show_fps = chkFps.checked;
  saveCurrentConfig();
});

chkAutostart.addEventListener('change', async () => {
  config.autostart = chkAutostart.checked;
  await invoke('set_autostart', { enable: config.autostart });
  saveCurrentConfig();
});

btnReset.addEventListener('click', async () => {
  try {
    config = await invoke<AppConfig>('reset_config');
    populateForm();
    await updateRadarMetrics();
    updateRadarDisplay();
  } catch (err) {
    console.error('Reset failed:', err);
  }
});

btnDone.addEventListener('click', async () => {
  try {
    const win = getCurrentWebviewWindow();
    await win.hide();
  } catch (err) {
    console.error('Failed to hide config window:', err);
  }
});

// Initialize Config Window
async function init() {
  try {
    const loadedConfig = await invoke<AppConfig>('get_config');
    if (loadedConfig) {
      config = loadedConfig;
    }
  } catch (err) {
    console.warn('Could not load config:', err);
  }

  await updateRadarMetrics();
  populateForm();
}

init();
