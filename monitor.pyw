import sys
import os
import psutil
import wmi
import ctypes
import json
import subprocess
import re
from PyQt5.QtWidgets import QApplication, QLabel, QVBoxLayout, QWidget, QFrame, QMessageBox, QSystemTrayIcon, QMenu, QAction, QStyle, QDialog, QSlider, QPushButton, QColorDialog, QFormLayout, QCheckBox
from PyQt5.QtCore import Qt, QTimer, QThread, pyqtSignal
from PyQt5.QtGui import QColor, QPainter, QPainterPath, QPen, QFont

def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

try:
    import pynvml
    pynvml.nvmlInit()
    nvml_available = True
except Exception:
    nvml_available = False

try:
    from pyadl import ADLManager
    adl_devices = ADLManager.getInstance().getDevices()
    adl_available = len(adl_devices) > 0
except Exception:
    adl_available = False

def get_cpu_temp(w):
    if not w:
        return "N/A"
    try:
        temperature_info = w.MSAcpi_ThermalZoneTemperature()
        if temperature_info:
            temp_k = temperature_info[0].CurrentTemperature
            return round((temp_k / 10.0) - 273.15, 1)
    except Exception:
        pass
    return "N/A"

def get_config_path():
    try:
        base_dir = os.path.dirname(os.path.abspath(__file__))
    except NameError:
        base_dir = os.path.dirname(os.path.abspath(sys.argv[0]))
    return os.path.join(base_dir, "config.json")

class PingThread(QThread):
    ping_result = pyqtSignal(str)

    def run(self):
        CREATE_NO_WINDOW = 0x08000000
        while True:
            try:
                output = subprocess.check_output(
                    "ping -n 1 -w 1000 8.8.8.8", 
                    shell=True, 
                    creationflags=CREATE_NO_WINDOW
                ).decode('utf-8', errors='ignore')
                
                # Match "time=25ms" or "เวลา=25ms"
                match = re.search(r'(?:time|เวลา|เวลาโดยประมาณ)[=<]\s*(\d+)\s*ms', output, re.IGNORECASE)
                if match:
                    self.ping_result.emit(f"Ping: {match.group(1)}ms")
                else:
                    self.ping_result.emit("Ping: Timeout")
            except Exception:
                self.ping_result.emit("Ping: Err")
            
            QThread.msleep(2000)

class OutlineLabel(QLabel):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.outline_color = QColor("black")
        self.text_color = QColor("#00ffcc")
        self.has_outline = False
        
    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.Antialiasing)
        
        path = QPainterPath()
        font = self.font()
        painter.setFont(font)
        
        metrics = painter.fontMetrics()
        # center vertically, left aligned
        y = (self.height() + metrics.ascent() - metrics.descent()) / 2
        path.addText(0, y, font, self.text())
        
        if self.has_outline:
            pen = QPen(self.outline_color)
            pen.setWidth(2)
            painter.setPen(pen)
            painter.drawPath(path)
            
        painter.setPen(self.text_color)
        painter.fillPath(path, self.text_color)

class PositionWindow(QDialog):
    def __init__(self, main_app):
        super().__init__()
        self.main_app = main_app
        self.setWindowFlags(Qt.WindowStaysOnTopHint | Qt.Tool)
        self.update_texts()

    def update_texts(self):
        if self.main_app.language == "en":
            self.setWindowTitle("Adjust Position")
        else:
            self.setWindowTitle("ตั้งค่าตำแหน่ง (Adjust Position)")

        screen = QApplication.primaryScreen().size()
        self.screen_width = screen.width()
        self.screen_height = screen.height()

        self.map_width = 400
        self.map_height = int(self.map_width * (self.screen_height / self.screen_width))
        
        self.setFixedSize(self.map_width + 40, self.map_height + 40)

        self.map_widget = QWidget(self)
        self.map_widget.setFixedSize(self.map_width, self.map_height)
        self.map_widget.setStyleSheet("background-color: #333; border: 1px solid #777;")
        self.map_widget.move(20, 20)

        self.app_rep = QWidget(self.map_widget)
        self.app_rep.setStyleSheet("background-color: #00ffcc;")
        
        self.update_rep_size()
        self.update_rep_pos()
        
        self.dragging = False

    def update_rep_size(self):
        w = int((self.main_app.width() / self.screen_width) * self.map_width)
        h = int((self.main_app.height() / self.screen_height) * self.map_height)
        self.app_rep.setFixedSize(max(w, 10), max(h, 10))

    def update_rep_pos(self):
        x = int((self.main_app.x() / self.screen_width) * self.map_width)
        y = int((self.main_app.y() / self.screen_height) * self.map_height)
        self.app_rep.move(x, y)

    def mousePressEvent(self, event):
        pos_in_map = self.map_widget.mapFrom(self, event.pos())
        if self.app_rep.geometry().contains(pos_in_map):
            self.dragging = True
            self.offset = pos_in_map - self.app_rep.pos()
        else:
            self.dragging = False

    def mouseMoveEvent(self, event):
        if self.dragging:
            pos_in_map = self.map_widget.mapFrom(self, event.pos())
            new_pos = pos_in_map - self.offset
            
            new_pos.setX(max(0, min(new_pos.x(), self.map_width - self.app_rep.width())))
            new_pos.setY(max(0, min(new_pos.y(), self.map_height - self.app_rep.height())))
            
            self.app_rep.move(new_pos)
            
            real_x = int((new_pos.x() / self.map_width) * self.screen_width)
            real_y = int((new_pos.y() / self.map_height) * self.screen_height)
            self.main_app.pos_x = real_x
            self.main_app.pos_y = real_y
            self.main_app.move(real_x, real_y)
            self.main_app.save_config()

    def mouseReleaseEvent(self, event):
        self.dragging = False

class ConfigWindow(QDialog):
    def __init__(self, main_app):
        super().__init__()
        self.main_app = main_app
        self.initUI()
        
    def initUI(self):
        self.setWindowFlags(Qt.WindowStaysOnTopHint | Qt.Tool)
        self.resize(350, 520)
        
        layout = QFormLayout()
        
        self.btn_lang_toggle = QPushButton()
        self.btn_lang_toggle.clicked.connect(self.toggle_language)
        layout.addRow(self.btn_lang_toggle)
        
        self.btn_position = QPushButton()
        self.btn_position.clicked.connect(self.adjust_position)
        layout.addRow(self.btn_position)

        # Color pickers
        self.btn_bg_color = QPushButton()
        self.btn_bg_color.clicked.connect(self.choose_bg_color)
        layout.addRow(self.btn_bg_color)
        
        self.btn_text_color = QPushButton()
        self.btn_text_color.clicked.connect(self.choose_text_color)
        layout.addRow(self.btn_text_color)
        
        self.btn_outline_color = QPushButton()
        self.btn_outline_color.clicked.connect(self.choose_outline_color)
        layout.addRow(self.btn_outline_color)
        
        # Opacity slider
        self.lbl_opacity = QLabel()
        self.slider_opacity = QSlider(Qt.Horizontal)
        self.slider_opacity.setRange(0, 255)
        self.slider_opacity.setValue(self.main_app.bg_alpha)
        self.slider_opacity.valueChanged.connect(self.change_opacity)
        layout.addRow(self.lbl_opacity, self.slider_opacity)
        # Font size slider
        self.lbl_font_size = QLabel()
        self.slider_font_size = QSlider(Qt.Horizontal)
        self.slider_font_size.setRange(6, 48)
        self.slider_font_size.setValue(self.main_app.font_size)
        self.slider_font_size.valueChanged.connect(self.change_font_size)
        layout.addRow(self.lbl_font_size, self.slider_font_size)

        
        # Toggles
        self.chk_outline = QCheckBox()
        self.chk_outline.setChecked(self.main_app.has_outline)
        self.chk_outline.stateChanged.connect(self.toggle_outline)
        layout.addRow(self.chk_outline)
        
        self.chk_cpu = QCheckBox()
        self.chk_cpu.setChecked(self.main_app.show_cpu)
        self.chk_cpu.stateChanged.connect(self.toggle_cpu)
        layout.addRow(self.chk_cpu)
        
        self.chk_gpu = QCheckBox()
        self.chk_gpu.setChecked(self.main_app.show_gpu)
        self.chk_gpu.stateChanged.connect(self.toggle_gpu)
        layout.addRow(self.chk_gpu)
        
        self.chk_ram = QCheckBox()
        self.chk_ram.setChecked(self.main_app.show_ram)
        self.chk_ram.stateChanged.connect(self.toggle_ram)
        layout.addRow(self.chk_ram)
        
        self.chk_disk = QCheckBox()
        self.chk_disk.setChecked(self.main_app.show_disk)
        self.chk_disk.stateChanged.connect(self.toggle_disk)
        layout.addRow(self.chk_disk)
        
        self.chk_net = QCheckBox()
        self.chk_net.setChecked(self.main_app.show_net)
        self.chk_net.stateChanged.connect(self.toggle_net)
        layout.addRow(self.chk_net)
        
        self.chk_ping = QCheckBox()
        self.chk_ping.setChecked(self.main_app.show_ping)
        self.chk_ping.stateChanged.connect(self.toggle_ping)
        layout.addRow(self.chk_ping)
        
        self.chk_fps = QCheckBox()
        self.chk_fps.setChecked(self.main_app.show_fps)
        self.chk_fps.stateChanged.connect(self.toggle_fps)
        layout.addRow(self.chk_fps)
        
        from PyQt5.QtWidgets import QComboBox
        self.combo_layout = QComboBox()
        self.combo_layout.addItems(["Vertical (แนวตั้ง)", "Horizontal (แนวนอน)"])
        if self.main_app.layout_direction == "horizontal":
            self.combo_layout.setCurrentIndex(1)
        self.combo_layout.currentIndexChanged.connect(self.change_layout_direction)
        layout.addRow("Layout:", self.combo_layout)

        
        self.chk_autostart = QCheckBox()
        self.chk_autostart.setChecked(self.main_app.autostart)
        self.chk_autostart.stateChanged.connect(self.toggle_autostart)
        layout.addRow(self.chk_autostart)
        
        self.btn_reset = QPushButton()
        self.btn_reset.setStyleSheet("background-color: #ff4d4d; color: white; font-weight: bold;")
        self.btn_reset.clicked.connect(self.reset_defaults)
        layout.addRow(self.btn_reset)
        
        self.setLayout(layout)
        self.update_texts()

    def toggle_language(self):
        self.main_app.language = "en" if self.main_app.language == "th" else "th"
        self.main_app.save_config()
        self.main_app.update_texts()

    def update_texts(self):
        lang = self.main_app.language
        if lang == "en":
            self.setWindowTitle("Config")
            self.btn_lang_toggle.setText("Language: EN")
            self.btn_position.setText("Adjust Position")
            self.btn_bg_color.setText("Choose Background Color")
            self.btn_text_color.setText("Choose Text Color")
            self.btn_outline_color.setText("Choose Outline Color")
            self.lbl_opacity.setText(f"Background Opacity ({self.slider_opacity.value()}):")
            self.lbl_font_size.setText(f"Font Size ({self.slider_font_size.value()}):")
            self.chk_outline.setText("Enable Text Outline")
            self.chk_cpu.setText("Show CPU")
            self.chk_gpu.setText("Show GPU")
            self.chk_ram.setText("Show RAM")
            self.chk_disk.setText("Show DISK (All Drives)")
            self.chk_net.setText("Show Network")
            self.chk_ping.setText("Show Ping (Latency)")
            self.chk_fps.setText("Show FPS")
            self.chk_autostart.setText("Run on Startup (Auto-Start)")
            self.btn_reset.setText("Reset to Defaults")
        else:
            self.setWindowTitle("ตั้งค่า (Config)")
            self.btn_lang_toggle.setText("Language: TH")
            self.btn_position.setText("ตั้งค่าตำแหน่ง (Adjust Position)")
            self.btn_bg_color.setText("เลือกสีพื้นหลัง (BG Color)")
            self.btn_text_color.setText("เลือกสีข้อความ (Text Color)")
            self.btn_outline_color.setText("เลือกสีขอบข้อความ (Outline Color)")
            self.lbl_opacity.setText(f"ความทึบพื้นหลัง ({self.slider_opacity.value()}):")
            self.lbl_font_size.setText(f"ขนาดตัวอักษร ({self.slider_font_size.value()}):")
            self.chk_outline.setText("เปิดใช้งานขอบข้อความ (Text Outline)")
            self.chk_cpu.setText("แสดง CPU")
            self.chk_gpu.setText("แสดง GPU")
            self.chk_ram.setText("แสดง RAM")
            self.chk_disk.setText("แสดง DISK (ทุกไดรฟ์)")
            self.chk_net.setText("แสดง Network")
            self.chk_ping.setText("แสดง Ping (Latency)")
            self.chk_fps.setText("แสดง FPS")
            self.chk_autostart.setText("เปิดโปรแกรมตอนเปิดเครื่อง (Auto-Start)")
            self.btn_reset.setText("คืนค่าเริ่มต้น (Reset to Defaults)")
        
    def adjust_position(self):
        self.pos_window = PositionWindow(self.main_app)
        self.pos_window.show()

    def choose_bg_color(self):
        self.hide()
        initial_color = QColor(self.main_app.bg_r, self.main_app.bg_g, self.main_app.bg_b)
        color = QColorDialog.getColor(initial_color, None, "เลือกสีพื้นหลัง")
        if color.isValid():
            self.main_app.bg_r = color.red()
            self.main_app.bg_g = color.green()
            self.main_app.bg_b = color.blue()
            self.main_app.apply_style()
            self.main_app.save_config()
        self.showNormal()
            
    def choose_text_color(self):
        self.hide()
        initial_color = self.main_app.text_color
        color = QColorDialog.getColor(initial_color, None, "เลือกสีข้อความ")
        if color.isValid():
            self.main_app.text_color = color
            self.main_app.apply_style()
            self.main_app.save_config()
        self.showNormal()
        
    def choose_outline_color(self):
        self.hide()
        initial_color = self.main_app.outline_color
        color = QColorDialog.getColor(initial_color, None, "เลือกสีขอบข้อความ")
        if color.isValid():
            self.main_app.outline_color = color
            self.main_app.apply_style()
            self.main_app.save_config()
        self.showNormal()
            
    def change_opacity(self, value):
        self.main_app.bg_alpha = value
        self.update_texts()
        self.main_app.apply_style()
        self.main_app.save_config()
        
    def change_font_size(self, value):
        self.main_app.font_size = value
        self.update_texts()
        self.main_app.apply_style()
        self.main_app.adjust_size()
        self.main_app.save_config()
        
    def toggle_outline(self, state):
        self.main_app.has_outline = (state == Qt.Checked)
        self.main_app.apply_style()
        self.main_app.save_config()

    def toggle_cpu(self, state):
        self.main_app.show_cpu = (state == Qt.Checked)
        self.main_app.lbl_cpu.setVisible(self.main_app.show_cpu)
        self.main_app.adjust_size()
        self.main_app.save_config()
        
    def toggle_gpu(self, state):
        self.main_app.show_gpu = (state == Qt.Checked)
        self.main_app.lbl_gpu.setVisible(self.main_app.show_gpu)
        self.main_app.adjust_size()
        self.main_app.save_config()
        
    def toggle_ram(self, state):
        self.main_app.show_ram = (state == Qt.Checked)
        self.main_app.lbl_ram.setVisible(self.main_app.show_ram)
        self.main_app.adjust_size()
        self.main_app.save_config()
        
    def toggle_disk(self, state):
        self.main_app.show_disk = (state == Qt.Checked)
        self.main_app.lbl_disk.setVisible(self.main_app.show_disk)
        self.main_app.adjust_size()
        self.main_app.save_config()
        
    def toggle_net(self, state):
        self.main_app.show_net = (state == Qt.Checked)
        self.main_app.lbl_net.setVisible(self.main_app.show_net)
        self.main_app.adjust_size()
        self.main_app.save_config()

    def toggle_ping(self, state):
        self.main_app.show_ping = (state == Qt.Checked)
        self.main_app.lbl_ping.setVisible(self.main_app.show_ping)
        self.main_app.adjust_size()
        self.main_app.save_config()
        
    def toggle_fps(self, state):
        self.main_app.show_fps = (state == Qt.Checked)
        self.main_app.lbl_fps.setVisible(self.main_app.show_fps)
        self.main_app.adjust_size()
        self.main_app.save_config()
        
    def change_layout_direction(self, index):
        self.main_app.layout_direction = "horizontal" if index == 1 else "vertical"
        self.main_app.rebuild_layout()
        self.main_app.save_config()

        
    def toggle_autostart(self, state):
        self.main_app.autostart = (state == Qt.Checked)
        self.main_app.apply_autostart()
        self.main_app.save_config()

    def reset_defaults(self):
        self.main_app.pos_x = 10
        self.main_app.pos_y = 10
        self.main_app.move(10, 10)
        
        self.main_app.bg_r = 0
        self.main_app.bg_g = 0
        self.main_app.bg_b = 0
        self.main_app.bg_alpha = 150
        self.main_app.font_size = 10
        
        self.main_app.text_color = QColor("#00ffcc")
        self.main_app.outline_color = QColor("black")
        self.main_app.has_outline = False
        
        self.main_app.show_cpu = True
        self.main_app.show_gpu = True
        self.main_app.show_ram = True
        self.main_app.show_disk = True
        self.main_app.show_net = True
        self.main_app.show_ping = True
        self.main_app.show_fps = False
        self.main_app.layout_direction = "vertical"
        self.combo_layout.setCurrentIndex(0)
        self.main_app.rebuild_layout()
        self.main_app.autostart = False
        self.main_app.apply_autostart()
        
        self.slider_opacity.blockSignals(True)
        self.slider_font_size.blockSignals(True)
        self.chk_outline.blockSignals(True)
        self.chk_cpu.blockSignals(True)
        self.chk_gpu.blockSignals(True)
        self.chk_ram.blockSignals(True)
        self.chk_disk.blockSignals(True)
        self.chk_net.blockSignals(True)
        self.chk_ping.blockSignals(True)
        self.chk_fps.blockSignals(True)
        self.chk_autostart.blockSignals(True)
        
        self.slider_opacity.setValue(150)
        self.slider_font_size.setValue(10)
        self.chk_outline.setChecked(False)
        self.chk_cpu.setChecked(True)
        self.chk_gpu.setChecked(True)
        self.chk_ram.setChecked(True)
        self.chk_disk.setChecked(True)
        self.chk_net.setChecked(True)
        self.chk_ping.setChecked(True)
        self.chk_fps.setChecked(False)
        self.chk_autostart.setChecked(False)
        
        self.slider_opacity.blockSignals(False)
        self.slider_font_size.blockSignals(False)
        self.chk_outline.blockSignals(False)
        self.chk_cpu.blockSignals(False)
        self.chk_gpu.blockSignals(False)
        self.chk_ram.blockSignals(False)
        self.chk_disk.blockSignals(False)
        self.chk_net.blockSignals(False)
        self.chk_ping.blockSignals(False)
        self.chk_fps.blockSignals(False)
        self.chk_autostart.blockSignals(False)
        
        self.main_app.lbl_cpu.setVisible(True)
        self.main_app.lbl_gpu.setVisible(True)
        self.main_app.lbl_ram.setVisible(True)
        self.main_app.lbl_disk.setVisible(True)
        self.main_app.lbl_net.setVisible(True)
        self.main_app.lbl_ping.setVisible(True)
        self.main_app.lbl_fps.setVisible(False)
        
        self.main_app.apply_style()
        self.main_app.adjust_size()
        self.main_app.save_config()

class MonitorApp(QWidget):
    def __init__(self, w, admin_status=""):
        super().__init__()
        self.w = w
        self.admin_status = admin_status
        
        # State
        self.language = "th"
        self.pos_x = 10
        self.pos_y = 10
        
        self.bg_r = 0
        self.bg_g = 0
        self.bg_b = 0
        self.bg_alpha = 150
        self.font_size = 10
        
        self.text_color = QColor("#00ffcc")
        self.outline_color = QColor("black")
        self.has_outline = False
        
        self.show_cpu = True
        self.show_gpu = True
        self.show_ram = True
        self.show_disk = True
        self.show_net = True
        self.show_ping = True
        self.show_fps = False
        self.layout_direction = "vertical"
        self.autostart = False
        
        self.load_config()
        
        self.net_io_last = psutil.net_io_counters()
        
        self.initUI()
        
        self.ping_thread = PingThread()
        self.ping_thread.ping_result.connect(self.update_ping_label)
        self.ping_thread.start()
        
        self.config_window = ConfigWindow(self)
        self.config_window.show()
        
    def load_config(self):
        try:
            with open(get_config_path(), "r", encoding="utf-8") as f:
                data = json.load(f)
                self.pos_x = data.get("pos_x", self.pos_x)
                self.pos_y = data.get("pos_y", self.pos_y)
                self.bg_r = data.get("bg_r", self.bg_r)
                self.bg_g = data.get("bg_g", self.bg_g)
                self.bg_b = data.get("bg_b", self.bg_b)
                self.bg_alpha = data.get("bg_alpha", self.bg_alpha)
                self.font_size = data.get("font_size", self.font_size)
                
                self.text_color = QColor(data.get("text_color", self.text_color.name()))
                self.outline_color = QColor(data.get("outline_color", self.outline_color.name()))
                self.has_outline = data.get("has_outline", self.has_outline)
                
                self.show_cpu = data.get("show_cpu", self.show_cpu)
                self.show_gpu = data.get("show_gpu", self.show_gpu)
                self.show_ram = data.get("show_ram", self.show_ram)
                self.show_disk = data.get("show_disk", data.get("show_ssd", self.show_disk))
                self.show_net = data.get("show_net", self.show_net)
                self.show_ping = data.get("show_ping", self.show_ping)
                self.show_fps = data.get("show_fps", self.show_fps)
                self.layout_direction = data.get("layout_direction", self.layout_direction)
                self.autostart = data.get("autostart", self.autostart)
                self.language = data.get("language", self.language)
        except Exception:
            pass

    def save_config(self):
        try:
            data = {
                "pos_x": self.x(),
                "pos_y": self.y(),
                "bg_r": self.bg_r,
                "bg_g": self.bg_g,
                "bg_b": self.bg_b,
                "bg_alpha": self.bg_alpha,
                "font_size": self.font_size,
                "text_color": self.text_color.name(),
                "outline_color": self.outline_color.name(),
                "has_outline": self.has_outline,
                "show_cpu": self.show_cpu,
                "show_gpu": self.show_gpu,
                "show_ram": self.show_ram,
                "show_disk": self.show_disk,
                "show_net": self.show_net,
                "show_ping": self.show_ping,
                "show_fps": self.show_fps,
                "layout_direction": self.layout_direction,
                "autostart": self.autostart,
                "language": self.language
            }
            with open(get_config_path(), "w", encoding="utf-8") as f:
                json.dump(data, f)
        except Exception as e:
            print("Error saving config:", e)

    def apply_autostart(self):
        startup_path = os.path.join(os.getenv('APPDATA'), r'Microsoft\Windows\Start Menu\Programs\Startup\MD Monitor.lnk')
        if self.autostart:
            try:
                target = os.path.abspath(__file__)
                pythonw = r"C:\Users\nong6\AppData\Local\Programs\Python\Python310\pythonw.exe"
                ps_cmd = f'''
                $WshShell = New-Object -comObject WScript.Shell
                $Shortcut = $WshShell.CreateShortcut("{startup_path}")
                $Shortcut.TargetPath = "{pythonw}"
                $Shortcut.Arguments = '"{target}"'
                $Shortcut.WorkingDirectory = "{os.path.dirname(target)}"
                $Shortcut.Save()
                '''
                CREATE_NO_WINDOW = 0x08000000
                subprocess.run(["powershell", "-Command", ps_cmd], creationflags=CREATE_NO_WINDOW)
            except Exception as e:
                print("Auto-start enable error:", e)
        else:
            try:
                if os.path.exists(startup_path):
                    os.remove(startup_path)
            except Exception as e:
                print("Auto-start disable error:", e)
            
    def apply_style(self):
        if hasattr(self, 'container'):
            self.container.setStyleSheet(f"#MainContainer {{ background-color: rgba({self.bg_r}, {self.bg_g}, {self.bg_b}, {self.bg_alpha}); border-radius: 10px; }}")
            
        labels = getattr(self, 'labels', [])
        for lbl in labels:
            lbl.text_color = self.text_color
            lbl.outline_color = self.outline_color
            lbl.has_outline = self.has_outline
            # transparent background for the label itself
            lbl.setStyleSheet("background-color: transparent;")
            lbl.setFont(QFont("Consolas", self.font_size, QFont.Bold))
            lbl.update()
            

    def rebuild_layout(self):
        from PyQt5.QtWidgets import QHBoxLayout, QVBoxLayout
        if self.container.layout() is not None:
            old_layout = self.container.layout()
            while old_layout.count():
                item = old_layout.takeAt(0)
                widget = item.widget()
                if widget:
                    widget.setParent(None)
            QWidget().setLayout(old_layout) # delete old layout
            
        if self.layout_direction == "horizontal":
            layout = QHBoxLayout(self.container)
        else:
            
            
        for lbl in self.labels:
            layout.addWidget(lbl)
            
        self.adjust_size()

    def adjust_size(self):

        self.resize(self.minimumSizeHint())
        
    def initUI(self):
        self.setWindowTitle("MD Monitor")
        
        # Frameless, Always on Top, Click-through
        self.setWindowFlags(Qt.WindowStaysOnTopHint | Qt.Tool | Qt.FramelessWindowHint | Qt.WindowTransparentForInput)
        self.setAttribute(Qt.WA_TranslucentBackground)
        
        self.container = QFrame(self)
        self.container.setObjectName("MainContainer")
        
        
        
        self.lbl_cpu = OutlineLabel("CPU: Loading...")
        self.lbl_cpu.setVisible(self.show_cpu)
        
        self.lbl_gpu = OutlineLabel("GPU: Loading...")
        self.lbl_gpu.setVisible(self.show_gpu)
        
        self.lbl_ram = OutlineLabel("RAM: Loading...")
        self.lbl_ram.setVisible(self.show_ram)
        
        self.lbl_disk = OutlineLabel("DISK: Loading...")
        self.lbl_disk.setVisible(self.show_disk)
        
        self.lbl_net = OutlineLabel("NET: Loading...")
        self.lbl_net.setVisible(self.show_net)
        
        self.lbl_ping = OutlineLabel("Ping: Loading...")
        self.lbl_ping.setVisible(self.show_ping)
        
        self.lbl_fps = OutlineLabel("FPS: N/A")
        self.lbl_fps.setVisible(self.show_fps)
        
        self.labels = [self.lbl_cpu, self.lbl_gpu, self.lbl_ram, self.lbl_disk, self.lbl_net, self.lbl_ping, self.lbl_fps]
        
        self.apply_style()
        
        self.rebuild_layout()
            
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.addWidget(self.container)
        
        if self.pos_x is not None and self.pos_y is not None:
            self.move(self.pos_x, self.pos_y)
            
        self.adjust_size()
        self.oldPos = self.pos()
        
        # --- System Tray Icon Setup ---
        self.tray_icon = QSystemTrayIcon(self)
        self.tray_icon.setIcon(self.style().standardIcon(QStyle.SP_ComputerIcon))
        self.tray_icon.setToolTip("MD Monitor")
        
        tray_menu = QMenu()
        self.show_action = QAction("แสดงหน้าจอ (Show)", self)
        self.show_action.triggered.connect(self.showNormal)
        
        self.hide_action = QAction("ซ่อนหน้าจอ (Hide)", self)
        self.hide_action.triggered.connect(self.hide)
        
        self.config_action = QAction("ตั้งค่า (Config)", self)
        self.config_action.triggered.connect(lambda: self.config_window.showNormal())
        
        self.quit_action = QAction("ปิดโปรแกรม (Exit)", self)
        self.quit_action.triggered.connect(QApplication.instance().quit)
        
        tray_menu.addAction(self.show_action)
        tray_menu.addAction(self.hide_action)
        tray_menu.addAction(self.config_action)
        tray_menu.addSeparator()
        tray_menu.addAction(self.quit_action)
        
        self.tray_icon.setContextMenu(tray_menu)
        self.tray_icon.show()
        
        QTimer.singleShot(100, self.update_texts)
        
        psutil.cpu_percent(interval=None)
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_stats)
        self.timer.start(1000)

    def mousePressEvent(self, event):
        self.oldPos = event.globalPos()

    def mouseMoveEvent(self, event):
        delta = event.globalPos() - self.oldPos
        self.move(self.x() + delta.x(), self.y() + delta.y())
        self.oldPos = event.globalPos()
        
    def mouseReleaseEvent(self, event):
        self.save_config()

    def mouseDoubleClickEvent(self, event):
        # Double click to close
        QApplication.instance().quit()

    def update_texts(self):
        if self.language == "en":
            self.show_action.setText("Show Monitor")
            self.hide_action.setText("Hide Monitor")
            self.config_action.setText("Config")
            self.quit_action.setText("Exit")
        else:
            self.show_action.setText("แสดงหน้าจอ (Show)")
            self.hide_action.setText("ซ่อนหน้าจอ (Hide)")
            self.config_action.setText("ตั้งค่า (Config)")
            self.quit_action.setText("ปิดโปรแกรม (Exit)")
            
        if hasattr(self, 'config_window') and self.config_window:
            self.config_window.update_texts()
            if hasattr(self.config_window, 'pos_window') and self.config_window.pos_window:
                self.config_window.pos_window.update_texts()

    def update_ping_label(self, text):
        if self.show_ping:
            self.lbl_ping.setText(text)
        
    def update_stats(self):
        try:
            if self.show_cpu:
                cpu_usage = psutil.cpu_percent(interval=None)
                cpu_temp_val = get_cpu_temp(self.w)
                cpu_temp = f"{cpu_temp_val}°C" if cpu_temp_val != "N/A" else f"N/A{self.admin_status}"
                self.lbl_cpu.setText(f"CPU: {cpu_usage}% | Temp: {cpu_temp}")
            
            if self.show_ram:
                ram = psutil.virtual_memory()
                self.lbl_ram.setText(f"RAM: {ram.percent}%")
            
            if self.show_disk:
                disk_strs = []
                for p in psutil.disk_partitions(all=False):
                    if p.fstype and 'cdrom' not in p.opts:
                        try:
                            usage = psutil.disk_usage(p.mountpoint)
                            drive_letter = p.mountpoint.replace(':\\', '').replace(':/', '')
                            disk_strs.append(f"{drive_letter} {usage.percent}%")
                        except Exception:
                            pass
                self.lbl_disk.setText("DISK: " + " | ".join(disk_strs))
            
            if self.show_gpu:
                gpu_usage = "N/A"
                gpu_temp = "N/A"
                gpu_fan = "N/A"
                if nvml_available:
                    try:
                        handle = pynvml.nvmlDeviceGetHandleByIndex(0)
                        util = pynvml.nvmlDeviceGetUtilizationRates(handle)
                        gpu_usage = util.gpu
                        gpu_temp = pynvml.nvmlDeviceGetTemperature(handle, pynvml.NVML_TEMPERATURE_GPU)
                        try:
                            gpu_fan = pynvml.nvmlDeviceGetFanSpeed(handle)
                        except:
                            pass
                    except Exception:
                        pass
                elif adl_available:
                    try:
                        dev = adl_devices[0]
                        gpu_usage = dev.getCurrentUsage()
                        try:
                            gpu_temp = dev.getCurrentTemperature()
                            gpu_fan = dev.getCurrentFanSpeed()
                        except:
                            pass
                    except Exception:
                        pass
                
                fan_str = f" | Fan: {gpu_fan}%" if gpu_fan != "N/A" else ""
                self.lbl_gpu.setText(f"GPU: {gpu_usage}% | Temp: {gpu_temp}°C{fan_str}")
                
            if self.show_net:
                net_io = psutil.net_io_counters()
                dl_bytes = net_io.bytes_recv - self.net_io_last.bytes_recv
                ul_bytes = net_io.bytes_sent - self.net_io_last.bytes_sent
                self.net_io_last = net_io
                
                def fmt_spd(b):
                    if b < 1024 * 1024:
                        return f"{b / 1024:.0f}K"
                    return f"{b / 1024 / 1024:.1f}M"
                    
                self.lbl_net.setText(f"NET: D {fmt_spd(dl_bytes)} | U {fmt_spd(ul_bytes)}")
            
        except Exception as e:
            print("Error updating stats:", e)

def log_msg(msg_text):
    try:
        log_dir = os.path.dirname(os.path.abspath(__file__))
    except NameError:
        log_dir = os.path.dirname(os.path.abspath(sys.argv[0]))
    log_path = os.path.join(log_dir, "app.log")
    with open(log_path, "a", encoding="utf-8") as f:
        f.write(msg_text + "\n")

def handle_exception(exc_type, exc_value, exc_traceback):
    import traceback
    err_msg = "".join(traceback.format_exception(exc_type, exc_value, exc_traceback))
    log_msg("CRASH:\n" + err_msg)
    sys.__excepthook__(exc_type, exc_value, exc_traceback)

sys.excepthook = handle_exception

def main():
    log_msg("=== App Starting ===")
    app = QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(False)
    
    if not is_admin():
        pythonw_path = r"C:\Users\nong6\AppData\Local\Programs\Python\Python310\pythonw.exe"
        script_path = os.path.abspath(sys.argv[0])
        log_msg(f"Not admin. Requesting UAC for: {pythonw_path} {script_path}")
        ctypes.windll.shell32.ShellExecuteW(None, "runas", pythonw_path, f'"{script_path}"', None, 1)
        sys.exit()
    
    log_msg("Running as admin. Initializing WMI...")
    admin_status = "[Admin]"
    try:
        w = wmi.WMI(namespace="root\\wmi")
        log_msg("WMI initialized successfully.")
    except Exception as e:
        w = None
        log_msg(f"WMI Error: {e}")
        admin_status = "[WMI Error]"
        
    log_msg("Creating MonitorApp...")
    ex = MonitorApp(w, admin_status)
    ex.show()
    
    # Save config on quit
    app.aboutToQuit.connect(ex.save_config)
    
    log_msg("MonitorApp shown. Entering event loop.")
    sys.exit(app.exec_())

if __name__ == '__main__':
    main()
