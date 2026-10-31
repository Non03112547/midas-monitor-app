import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface DiskTelemetry {
  name: string;
  mount_point: string;
  usage_percent: number;
}

interface TelemetryUpdate {
  cpu_usage: number;
  cpu_temp: number;
  ram_usage: number;
  gpu_usage: number;
  gpu_temp: number;
  net_down_kbps: number;
  net_up_kbps: number;
  ping_ms: number;
  disks: DiskTelemetry[];
  disk_summary: string;
}

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

// DOM Elements
const osdContainer = document.getElementById('osd-container')!;
const rowCpu = document.getElementById('row-cpu')!;
const rowGpu = document.getElementById('row-gpu')!;
const rowRam = document.getElementById('row-ram')!;
const rowDisk = document.getElementById('row-disk')!;
const rowNetDown = document.getElementById('row-net-down')!;
const rowNetUp = document.getElementById('row-net-up')!;
const rowPing = document.getElementById('row-ping')!;
const rowFps = document.getElementById('row-fps')!;

const cpuVal = document.getElementById('cpu-val')!;
const ramVal = document.getElementById('ram-val')!;
const gpuVal = document.getElementById('gpu-val')!;
const gpuTemp = document.getElementById('gpu-temp')!;
const diskVal = document.getElementById('disk-val')!;
const netDown = document.getElementById('net-down')!;
const netUp = document.getElementById('net-up')!;
const pingVal = document.getElementById('ping-val')!;
const fpsVal = document.getElementById('fps-val')!;

let config: AppConfig = {
  pos_x: 28,
  pos_y: 19,
  bg_r: 0,
  bg_g: 0,
  bg_b: 0,
  bg_alpha: 60,
  font_size: 20,
  text_color: "#ffffff",
  outline_color: "#000000",
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

function autoFitWindow() {
  requestAnimationFrame(() => {
    const rect = osdContainer.getBoundingClientRect();
    const width = Math.max(120, Math.ceil(rect.width) + 30);
    const height = Math.max(40, Math.ceil(rect.height) + 20);
    invoke('set_window_size', { width, height }).catch(() => {});
  });
}

function applyOsdStyle() {
  const root = document.documentElement;
  root.style.setProperty('--font-size', `${config.font_size}px`);
  root.style.setProperty('--text-color', config.text_color);
  root.style.setProperty('--outline-color', config.outline_color);
  root.style.setProperty('--bg-r', config.bg_r.toString());
  root.style.setProperty('--bg-g', config.bg_g.toString());
  root.style.setProperty('--bg-b', config.bg_b.toString());
  root.style.setProperty('--bg-alpha', config.bg_alpha.toString());

  // Outline
  if (config.has_outline) {
    osdContainer.classList.add('has-outline');
  } else {
    osdContainer.classList.remove('has-outline');
  }

  // Layout Direction
  if (config.layout_direction === 'horizontal') {
    osdContainer.classList.add('layout-horizontal');
  } else {
    osdContainer.classList.remove('layout-horizontal');
  }

  // Visibility of Rows
  rowCpu.style.display = config.show_cpu ? 'flex' : 'none';
  rowGpu.style.display = config.show_gpu ? 'flex' : 'none';
  rowRam.style.display = config.show_ram ? 'flex' : 'none';
  rowDisk.style.display = config.show_disk ? 'flex' : 'none';
  rowNetDown.style.display = config.show_net ? 'flex' : 'none';
  rowNetUp.style.display = config.show_net ? 'flex' : 'none';
  rowPing.style.display = config.show_ping ? 'flex' : 'none';
  rowFps.style.display = config.show_fps ? 'flex' : 'none';

  autoFitWindow();
}

// Telemetry Listener
listen<TelemetryUpdate>('telemetry-update', (event) => {
  const data = event.payload;
  const cpuTempStr = data.cpu_temp > 0 ? `${data.cpu_temp.toFixed(1)}°C` : 'N/A';
  cpuVal.textContent = `${data.cpu_usage.toFixed(1)}% | Temp: ${cpuTempStr}`;
  ramVal.textContent = `${data.ram_usage.toFixed(1)}%`;
  gpuVal.textContent = `${data.gpu_usage.toFixed(1)}%`;
  gpuTemp.textContent = data.gpu_temp > 0 ? `${data.gpu_temp.toFixed(1)}°C` : 'N/A';
  netDown.textContent = `${data.net_down_kbps.toFixed(1)} KB/s`;
  netUp.textContent = `${data.net_up_kbps.toFixed(1)} KB/s`;
  pingVal.textContent = `${data.ping_ms.toFixed(1)} ms`;
  diskVal.textContent = data.disk_summary || '--';
  fpsVal.textContent = 'N/A';
  autoFitWindow();
}).catch(console.error);

// Live Config Updates Listener
listen<AppConfig>('config-changed', (event) => {
  if (event.payload) {
    config = event.payload;
    applyOsdStyle();
  }
}).catch(console.error);

// Initialize OSD
async function init() {
  try {
    const loadedConfig = await invoke<AppConfig>('get_config');
    if (loadedConfig) {
      config = loadedConfig;
    }
  } catch (err) {
    console.warn('Could not load config:', err);
  }

  applyOsdStyle();
  await invoke('set_ignore_cursor', { ignore: true }).catch(() => {});
}

init();
