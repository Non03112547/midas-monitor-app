# ⚡ MD Monitor (Midas Monitor)

<p align="center">
  <img src="app-icon.png" alt="MD Monitor Logo" width="128" height="128" style="border-radius: 24px; box-shadow: 0 4px 20px rgba(0,0,0,0.15);" />
</p>

<p align="center">
  <b>ซอฟต์แวร์ On-Screen Display (OSD) ตรวจสอบสถานะฮาร์ดแวร์และพลังงานแบบเรียลไทม์</b><br>
  น้ำหนักเบาเป็นพิเศษ (0% CPU Overhead) พัฒนาด้วย <b>Tauri 2</b>, <b>Rust Engine</b> และ <b>TypeScript</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-1.1.0-gold.svg" alt="Version 1.1.0" />
  <img src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-blue.svg" alt="Platform Windows" />
  <img src="https://img.shields.io/badge/framework-Tauri%202-orange.svg" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/backend-Rust-black.svg" alt="Rust" />
  <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License MIT" />
</p>

---

## 📖 เกี่ยวกับโปรเจกต์ (About Project)

**MD Monitor** คือแอปพลิเคชันยูทิลิตีสำหรับระบบปฏิบัติการ Windows ออกแบบมาเพื่อให้นักเล่นเกม ครีเอเตอร์ และผู้ใช้งานระดับสูง สามารถมองเห็นข้อมูลสถานะฮาร์ดแวร์แบบเรียลไทม์ผ่านแถบแสดงผล On-Screen Display (OSD) สไตล์ Minimalist / Cyberpunk โดยไม่บดบังสายตา และไม่กินพลังประมวลผลของเครื่องคอมพิวเตอร์

---

## ✨ คุณสมบัติเด่น (Key Features)

- ⚡ **Universal Multi-Vendor GPU Telemetry:** แสดงผลอัตราการทำงาน (%) และอุณหภูมิความร้อน (°C) แบบเรียลไทม์ รองรับการ์ดจอทุกค่าย: **NVIDIA** (ผ่าน NVML), **AMD** (ผ่าน ADL) และ **Intel Arc / Iris Xe / UHD** (ผ่าน Windows Universal WDDM / PDH Engine) ครอบคลุม 100% ของการ์ดจอในตลาด
- 🧠 **RAM Monitoring:** รายงานปริมาณการใช้งานแรมแบบแม่นยำ
- 💾 **Unified DISK Monitoring:** รวมสัดส่วนพื้นที่จัดเก็บข้อมูลไดรฟ์ระบบและไดรฟ์ข้อมูล (C + D) แบบรวบยอด
- 🔋 **Hardware Power Consumption (W & Wh):** ตรวจวัดอัตราการกินไฟของฮาร์ดแวร์เป็นวัตต์ (Watt) และคำนวณพลังงานสะสม (Watt-hour) ผ่าน Windows PDH API
- 🌐 **Network Latency & Ping:** วัดค่า Ping / Latency และความเร็วรับส่งข้อมูลเครือข่ายอย่างต่อเนื่อง
- 🎯 **Hybrid Two-Tier FPS Engine:**
  - **In-Game Mode:** ตรวจจับเฟรมเรตในเกมระดับฮาร์ดแวร์ผ่าน AMD ADL FrameMetrics
  - **Desktop Fallback:** เมื่ออยู่นอกเกม ระบบจะสลับมาแสดง Display Refresh Rate (เช่น 60 FPS) อัตโนมัติ ป้องกันปัญหาขึ้น N/A

---

## 🚀 วิธีการติดตั้ง (Installation)

### ทางเลือกที่ 1: ติดตั้งผ่านตัวติดตั้งสำเร็จรูป (แนะนำสำหรับผู้ใช้ทั่วไป)
1. ดาวน์โหลดไฟล์ติดตั้งล่าสุด `MD-Monitor-Setup.exe` ได้จาก:
   - [Midas Web Showcase Portal](http://localhost:3001/products/md-monitor)
   - หรือหน้า [GitHub Releases](https://github.com/Non03112547/midas-monitor-app/releases)
2. ดับเบิลคลิกไฟล์ `MD-Monitor-Setup.exe` แล้วทำตามขั้นตอนการติดตั้งของ NSIS Installer
3. เมื่อติดตั้งเสร็จสิ้น โปรแกรมจะสร้าง Shortcut บนเดสก์ท็อปและเปิดใช้งานได้ทันที

### ทางเลือกที่ 2: คอมไพล์จากโค้ดต้นฉบับ (Build from Source)
**ความต้องการของระบบสำหรับการพัฒนา:**
- [Node.js](https://nodejs.org/) (เวอร์ชัน 18 ขึ้นไป)
- [Rust Toolchain](https://rustup.rs/) (stable-x86_64-pc-windows-msvc)
- [C++ Build Tools สำหรับ Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

**ขั้นตอนการบิลด์:**
```bash
# โคลนโปรเจกต์
git clone -b tauri https://github.com/Non03112547/midas-monitor-app.git
cd midas-monitor-app

# ติดตั้งแพ็กเกจ Frontend
npm install

# รันในโหมด Development
npm run tauri dev

# คอมไพล์เป็น Production Installer (NSIS .exe)
npm run tauri build
```
ไฟล์ติดตั้งจะอยู่ที่: `src-tauri/target/release/bundle/nsis/MD-Monitor_1.0.0_x64-setup.exe`

---

## 🕹️ คู่มือและวิธีการใช้งาน (User Guide)

### 1. หน้าต่างแสดงผล OSD (Main Overlay)
- เมื่อเปิดโปรแกรม แถบตัวเลข OSD จะปรากฏบนหน้าจอตามตำแหน่งที่กำหนดไว้
- การแสดงผลจัดวางแบบชิดซ้าย เป็นระเบียบ อ่านค่าง่าย
- มีความโปร่งใสและแสดงผลซ้อนทับอยู่ด้านบนเสมอ (Always on Top) เพื่อให้มองเห็นระหว่างทำงานหรือเล่นเกม

### 2. การเปิดหน้าต่างตั้งค่า (Settings & Configuration)
- สามารถเปิดหน้าต่างตั้งค่าได้ผ่านถาดระบบ (System Tray) หรือคลิกขวาที่ตัว OSD
- **การปรับตำแหน่ง (Positioning):**
  - กดที่หัวข้อ **"ปรับตำแหน่งการแสดงผล (Screen Position)"** กล่องเรดาร์จำลองตำแหน่งหน้าจอจะกางออกมา
  - เลื่อนแถบ Slider แนวนอน (X) และแนวตั้ง (Y) เพื่อจัดวาง OSD ในมุมที่ต้องการ
  - เมื่อปรับเสร็จแล้ว สามารถกดคลิกที่หัวข้ออีกครั้งเพื่อ **พับซ่อน (Collapse)** กล่องปรับตำแหน่งกลับเข้าไปได้ทันที
- **การเปิด/ปิดเซนเซอร์ที่ต้องการแสดง:**
  - สลับสวิตช์เปิด-ปิดของแต่ละโมดูล (CPU, GPU, RAM, DISK, PWR, PING, FPS) ตามความต้องการ
- **การเลือกธีมสีและขนาด:**
  - ปรับขนาดตัวอักษร ความโปร่งใสของพื้นหลัง และชุดสีที่ชื่นชอบ

---

## 🏗️ โครงสร้างสถาปัตยกรรม (Architecture)

```text
midas-monitor-app/
├── src-tauri/                  # แกนประมวลผลระดับระบบ (Rust Engine)
│   ├── src/
│   │   ├── main.rs             # Tauri Entrypoint & IPC Handlers
│   │   ├── models.rs           # โครงสร้างข้อมูล TelemetryData
│   │   ├── system_info.rs      # การอ่านค่า CPU, RAM, Disk, Network
│   │   ├── fps.rs              # Hybrid FPS Engine (AMD ADL + DWM/EnumDisplaySettings)
│   │   ├── power.rs            # Windows PDH Power Hardware Sampling
│   │   └── gpu_adl.rs          # AMD ADL GPU Metrics & Temperature
│   ├── Cargo.toml              # การตั้งค่า Rust Dependencies
│   └── tauri.conf.json         # การตั้งค่าหน้าต่าง OSD & Config Window
├── src/                        # ส่วนติดต่อผู้ใช้ (Frontend Webview)
│   ├── main.ts                 # ตรรกะการรับข้อมูล IPC และเรนเดอร์ OSD
│   └── config.ts               # ตรรกะหน้าต่างตั้งค่าและบันทึก Preferences
├── config.html                 # หน้าต่างตั้งค่า (Settings UI)
├── index.html                  # หน้าต่าง OSD หลัก (Main OSD View)
├── package.json                # สคริปต์ Node.js และ Vite Dependencies
└── README.md                   # เอกสารประกอบและคู่มือการใช้งาน
```

---

## 🛡️ เทคโนโลยีที่ใช้ (Tech Stack)

- **Engine:** [Tauri 2](https://v2.tauri.app/) (Rust-based ultra-lightweight desktop runtime)
- **Frontend:** HTML5, CSS3 (Modern Glassmorphism), TypeScript, Vite
- **Low-Level Native APIs:**
  - `AMD ADL (AMD Display Library)`: ตรวจวัด GPU Telemetry & FrameMetrics
  - `Windows PDH (Performance Data Helper)`: ตรวจวัดอัตราการใช้พลังงานฮาร์ดแวร์ (Watts & Watt-hours)
  - `Win32 API (User32 / DWM)`: ตรวจจับอัตราการรีเฟรชหน้าจอ (Display Refresh Rate) และควบคุมเลเยอร์หน้าต่าง

---

## 📄 ลิขสิทธิ์ (License)

โปรเจกต์นี้เผยแพร่ภายใต้ลิขสิทธิ์ [MIT License](LICENSE) — พัฒนาและดูแลโดยทีมงาน **Midas**
