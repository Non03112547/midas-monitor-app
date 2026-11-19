import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface DailyRecord {
  date: string;
  duration_sec: number;
  sample_count: number;
  avg_cpu_usage: number;
  avg_cpu_temp: number;
  avg_gpu_usage: number;
  avg_gpu_temp: number;
  avg_ram_usage: number;
  avg_disk_usage: number;
  avg_power_w: number;
  total_energy_wh: number;
}

interface DailyHistoryData {
  first_used_date: string | null;
  days: DailyRecord[];
}

interface AppConfig {
  language?: string;
}

const i18n: Record<string, Record<string, string>> = {
  th: {
    pageTitle: 'ประวัติการใช้งานรายวัน',
    pageSubtitle: 'บันทึกค่าเฉลี่ยรายวันและการใช้ไฟสะสมตั้งแต่วันแรกที่ใช้งาน',
    btnRefresh: 'รีเฟรช',
    btnClear: 'ล้างประวัติ',
    lblFirstUsed: 'เริ่มใช้งานครั้งแรก',
    lblTotalDays: 'จำนวนวันทั้งหมด',
    lblTotalTime: 'เวลารวมทั้งหมด',
    lblTotalEnergy: 'การใช้ไฟสะสมรวม',
    unitDay: 'วัน',
    unitHr: 'ชม.',
    unitMin: 'นาที',
    unitSec: 'วินาที',
    badgeToday: 'วันนี้',
    empty: 'ยังไม่มีข้อมูลประวัติการใช้งานรายวัน',
    confirmClear: 'ต้องการล้างประวัติการใช้งานรายวันทั้งหมดหรือไม่? (ข้อมูลจะถูกลบถาวร)',
    lblCpu: 'CPU เฉลี่ย',
    lblGpu: 'GPU เฉลี่ย',
    lblRamDisk: 'RAM / Disk เฉลี่ย',
    lblPower: 'กำลังไฟเฉลี่ย',
    lblEnergy: 'ใช้ไฟสะสมทั้งวัน',
  },
  en: {
    pageTitle: 'Daily Usage History',
    pageSubtitle: 'Daily average telemetry and cumulative energy usage since first use',
    btnRefresh: 'Refresh',
    btnClear: 'Clear History',
    lblFirstUsed: 'First Used Date',
    lblTotalDays: 'Total Days',
    lblTotalTime: 'Total Uptime',
    lblTotalEnergy: 'Cumulative Energy',
    unitDay: 'days',
    unitHr: 'h',
    unitMin: 'm',
    unitSec: 's',
    badgeToday: 'Today',
    empty: 'No daily usage history recorded yet',
    confirmClear: 'Clear all daily usage history? This action cannot be undone.',
    lblCpu: 'Avg CPU',
    lblGpu: 'Avg GPU',
    lblRamDisk: 'Avg RAM / Disk',
    lblPower: 'Avg Power',
    lblEnergy: 'Daily Energy',
  },
};

let currentLang = 'th';
let cachedData: DailyHistoryData = { first_used_date: null, days: [] };

const pageTitleEl = document.getElementById('page-title')!;
const pageSubtitleEl = document.getElementById('page-subtitle')!;
const txtBtnRefresh = document.getElementById('txt-btn-refresh')!;
const txtBtnClear = document.getElementById('txt-btn-clear')!;
const lblFirstUsed = document.getElementById('lbl-first-used')!;
const valFirstUsed = document.getElementById('val-first-used')!;
const lblTotalDays = document.getElementById('lbl-total-days')!;
const valTotalDays = document.getElementById('val-total-days')!;
const lblTotalTime = document.getElementById('lbl-total-time')!;
const valTotalTime = document.getElementById('val-total-time')!;
const lblTotalEnergy = document.getElementById('lbl-total-energy')!;
const valTotalEnergy = document.getElementById('val-total-energy')!;
const dailyList = document.getElementById('daily-list')!;
const btnRefresh = document.getElementById('btn-refresh')!;
const btnClear = document.getElementById('btn-clear')!;

function getDict() {
  return i18n[currentLang] || i18n.th;
}

function formatDuration(totalSec: number): string {
  const dict = getDict();
  const hrs = Math.floor(totalSec / 3600);
  const mins = Math.floor((totalSec % 3600) / 60);
  const secs = Math.floor(totalSec % 60);
  if (hrs > 0) {
    return `${hrs} ${dict.unitHr} ${mins} ${dict.unitMin}`;
  }
  if (mins > 0) {
    return `${mins} ${dict.unitMin} ${secs} ${dict.unitSec}`;
  }
  return `${secs} ${dict.unitSec}`;
}

function formatEnergy(wh: number): string {
  if (wh >= 1000) {
    return `${(wh / 1000).toFixed(2)} kWh`;
  }
  return `${wh.toFixed(1)} Wh`;
}

function getLocalTodayString(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, '0');
  const d = String(now.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

function applyLabels() {
  const dict = getDict();
  document.title = `${dict.pageTitle} - MD Monitor`;
  pageTitleEl.textContent = dict.pageTitle;
  pageSubtitleEl.textContent = dict.pageSubtitle;
  txtBtnRefresh.textContent = dict.btnRefresh;
  txtBtnClear.textContent = dict.btnClear;
  lblFirstUsed.textContent = dict.lblFirstUsed;
  lblTotalDays.textContent = dict.lblTotalDays;
  lblTotalTime.textContent = dict.lblTotalTime;
  lblTotalEnergy.textContent = dict.lblTotalEnergy;
}

function render() {
  applyLabels();
  const dict = getDict();
  const days = cachedData.days || [];

  const totalSec = days.reduce((acc, d) => acc + (d.duration_sec || 0), 0);
  const totalWh = days.reduce((acc, d) => acc + (d.total_energy_wh || 0), 0);

  valFirstUsed.textContent = cachedData.first_used_date || '--';
  valTotalDays.textContent = `${days.length} ${dict.unitDay}`;
  valTotalTime.textContent = formatDuration(totalSec);
  valTotalEnergy.textContent = formatEnergy(totalWh);

  dailyList.innerHTML = '';
  if (days.length === 0) {
    const empty = document.createElement('div');
    empty.className = 'empty-state';
    empty.textContent = dict.empty;
    dailyList.appendChild(empty);
    return;
  }

  const todayStr = getLocalTodayString();

  for (const day of days) {
    const isToday = day.date === todayStr;
    const card = document.createElement('article');
    card.className = `daily-card${isToday ? ' today' : ''}`;

    const cpuTemp = day.avg_cpu_temp > 0 ? `${day.avg_cpu_temp.toFixed(1)}°C` : '--';
    const gpuTemp = day.avg_gpu_temp > 0 ? `${day.avg_gpu_temp.toFixed(1)}°C` : '--';

    card.innerHTML = `
      <div class="daily-card-header">
        <div class="daily-date-wrap">
          <span class="daily-date">${day.date}</span>
          ${isToday ? `<span class="badge-today">${dict.badgeToday}</span>` : ''}
        </div>
        <span class="daily-duration">${formatDuration(day.duration_sec)}</span>
      </div>
      <div class="daily-metrics">
        <div class="metric-box">
          <span class="metric-label">${dict.lblCpu}</span>
          <span class="metric-val">${day.avg_cpu_usage.toFixed(1)}% (${cpuTemp})</span>
        </div>
        <div class="metric-box">
          <span class="metric-label">${dict.lblGpu}</span>
          <span class="metric-val">${day.avg_gpu_usage.toFixed(1)}% (${gpuTemp})</span>
        </div>
        <div class="metric-box">
          <span class="metric-label">${dict.lblRamDisk}</span>
          <span class="metric-val">${day.avg_ram_usage.toFixed(0)}% / ${day.avg_disk_usage.toFixed(0)}%</span>
        </div>
        <div class="metric-box">
          <span class="metric-label">${dict.lblPower}</span>
          <span class="metric-val">${day.avg_power_w.toFixed(1)} W</span>
        </div>
        <div class="metric-box energy">
          <span class="metric-label">${dict.lblEnergy}</span>
          <span class="metric-val">${formatEnergy(day.total_energy_wh)}</span>
        </div>
      </div>
    `;
    dailyList.appendChild(card);
  }
}

async function loadDailyHistory() {
  try {
    cachedData = await invoke<DailyHistoryData>('get_daily_history');
    render();
  } catch (err) {
    console.error('Failed to load daily history:', err);
  }
}

btnRefresh.addEventListener('click', () => {
  void loadDailyHistory();
});

btnClear.addEventListener('click', async () => {
  const dict = getDict();
  if (!window.confirm(dict.confirmClear)) {
    return;
  }
  try {
    await invoke('clear_daily_history');
    await loadDailyHistory();
  } catch (err) {
    console.error('Failed to clear daily history:', err);
  }
});

listen<AppConfig>('config-changed', (event) => {
  if (event.payload?.language) {
    currentLang = event.payload.language;
    render();
  }
}).catch(console.error);

async function init() {
  try {
    const cfg = await invoke<AppConfig>('get_config');
    if (cfg?.language) {
      currentLang = cfg.language;
    }
  } catch (err) {
    console.warn('Could not load config for history window:', err);
  }
  await loadDailyHistory();
}

void init();
