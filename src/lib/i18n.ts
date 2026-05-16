import { derived } from 'svelte/store';
import { language } from './stores/settings';

export type Lang = 'zh' | 'en';

const translations: Record<string, Record<Lang, string>> = {
  // App
  'app.title': { zh: 'ADBQTools', en: 'ADBQTools' },

  // Toolbar
  'toolbar.logcat': { zh: 'Logcat', en: 'Logcat' },
  'toolbar.installer': { zh: '安装器', en: 'Installer' },

  // Device Panel
  'devices.label': { zh: '设备', en: 'Devices' },
  'devices.none': { zh: '无设备', en: 'No devices' },
  'devices.wifi_connect': { zh: 'WiFi 连接', en: 'WiFi Connect' },
  'devices.restart_adb': { zh: '重启 ADB (kill-server + start-server)', en: 'Restart ADB (kill-server + start-server)' },
  'devices.disconnect': { zh: '断开连接', en: 'Disconnect' },

  // WiFi Dialog
  'wifi.title': { zh: 'WiFi 连接', en: 'WiFi Connection' },
  'wifi.connect': { zh: '连接', en: 'Connect' },
  'wifi.pair': { zh: '配对', en: 'Pair' },
  'wifi.connected': { zh: '已连接', en: 'Connected' },
  'wifi.paired': { zh: '已配对', en: 'Paired' },
  'wifi.connecting': { zh: '连接中...', en: 'Connecting...' },
  'wifi.pairing': { zh: '配对中...', en: 'Pairing...' },
  'wifi.pair_address': { zh: '配对地址', en: 'Pair Address' },
  'wifi.device_address': { zh: '设备地址', en: 'Device Address' },
  'wifi.pairing_code': { zh: '配对码', en: 'Pairing Code' },
  'wifi.saved_ips': { zh: '已保存的地址', en: 'Saved Addresses' },
  'wifi.connect_desc': { zh: '连接已配对的设备。在手机 设置 → 开发者选项 → 无线调试 页面查看连接地址和端口。', en: 'Connect to an already-paired device. Find the connection address and port on your phone under Settings → Developer Options → Wireless Debugging.' },
  'wifi.pair_desc': { zh: '首次连接需要先配对。在手机 设置 → 开发者选项 → 无线调试 → 使用配对码配对设备，输入显示的配对地址和6位配对码。配对成功后，再使用 Connect 连接。', en: 'First-time connection requires pairing. On your phone, go to Settings → Developer Options → Wireless Debugging → Pair device with pairing code. Enter the pairing address and 6-digit code shown. After pairing, use Connect to establish the connection.' },

  // Logcat Filter Bar
  'logcat.filter_placeholder': { zh: '过滤日志... (tag:, pid:, message:)', en: 'Filter logcat... (tag:, pid:, message:)' },
  'logcat.regex_toggle': { zh: '正则模式', en: 'Toggle regex mode' },
  'logcat.all_levels': { zh: '显示所有日志级别', en: 'Show all log levels' },
  'logcat.show_above': { zh: '显示 {level} 及以上', en: 'Show {level} and above' },
  'logcat.unity_filter': { zh: 'Unity 过滤 — 仅显示 Unity, IL2CPP, Mono 日志', en: 'Unity filter — show only Unity, IL2CPP, Mono logs' },
  'logcat.soft_wrap': { zh: '自动换行 — 折行显示长文本', en: 'Soft-Wrap — wrap long lines' },
  'logcat.scroll_end': { zh: '滚动到底部 — 自动跟踪新日志', en: 'Scroll to end — auto-follow new logs' },
  'logcat.resume': { zh: '继续 — 继续接收日志', en: 'Resume — continue receiving logs' },
  'logcat.pause': { zh: '暂停 — 停止接收新日志', en: 'Pause — stop receiving new logs' },
  'logcat.clear': { zh: '清除 — 删除所有日志', en: 'Clear — delete all log entries' },
  'logcat.export': { zh: '导出 — 保存日志到 .txt 文件', en: 'Export — save logs to .txt file' },
  'logcat.waiting': { zh: '等待日志数据...', en: 'Waiting for logcat data...' },
  'logcat.select_device': { zh: '请选择设备', en: 'Select a device to start' },
  'logcat.no_match': { zh: '没有匹配当前过滤条件的日志', en: 'No entries match current filters' },

  // Logcat header
  'logcat.hdr_timestamp': { zh: '时间戳', en: 'Timestamp' },
  'logcat.hdr_pidtid': { zh: 'PID-TID', en: 'PID-TID' },
  'logcat.hdr_package': { zh: '包名', en: 'Package' },
  'logcat.hdr_tag': { zh: '标签', en: 'Tag' },
  'logcat.hdr_level': { zh: '级别', en: 'Lvl' },
  'logcat.hdr_message': { zh: '消息', en: 'Message' },

  // Query suggestions
  'suggest.tag': { zh: '按日志标签过滤', en: 'Filter by log tag' },
  'suggest.pid': { zh: '按进程 ID 过滤', en: 'Filter by process ID' },
  'suggest.tid': { zh: '按线程 ID 过滤', en: 'Filter by thread ID' },
  'suggest.message': { zh: '按消息内容过滤', en: 'Filter by message content' },
  'suggest.level': { zh: '按最低级别过滤', en: 'Filter by min level' },
  'suggest.package': { zh: '按包名过滤', en: 'Filter by package name' },

  // Installer
  'installer.drag_hint': { zh: '拖拽 APK / AAB 文件到这里', en: 'Drag APK / AAB here' },
  'installer.browse_hint': { zh: '或点击浏览文件', en: 'or click to browse files' },
  'installer.browse': { zh: '浏览文件', en: 'Browse Files' },
  'installer.signing': { zh: '签名', en: 'Signing' },
  'installer.signing_config': { zh: '签名配置', en: 'Signing Configuration' },
  'installer.install': { zh: '安装', en: 'Install' },
  'installer.installing': { zh: '安装中...', en: 'Installing...' },
  'installer.success': { zh: '安装成功', en: 'Installed successfully' },
  'installer.keystore_file': { zh: 'Keystore 文件', en: 'Keystore File' },
  'installer.store_password': { zh: 'Store 密码', en: 'Store Password' },
  'installer.key_password': { zh: 'Key 密码', en: 'Key Password' },
  'installer.key_alias': { zh: 'Key 别名', en: 'Key Alias' },
  'installer.select_keystore': { zh: '选择 keystore 文件...', en: 'Select keystore file...' },
  'installer.reading_aliases': { zh: '读取别名中...', en: 'Reading aliases...' },
  'installer.select_alias': { zh: '选择别名...', en: 'Select an alias...' },
  'installer.alias_hint': { zh: '先选择 keystore 文件并输入 store 密码', en: 'Select keystore file & enter store password first' },
  'installer.remove': { zh: '移除', en: 'Remove' },

  // Install History
  'history.title': { zh: '安装记录', en: 'Install History' },
  'history.clear': { zh: '清空', en: 'Clear' },
  'history.empty': { zh: '暂无安装记录', en: 'No install records yet' },
  'history.success': { zh: '成功', en: 'Success' },
  'history.failed': { zh: '失败', en: 'Failed' },

  // Settings
  'settings.title': { zh: '设置', en: 'Settings' },
  'settings.language': { zh: '语言设置', en: 'Language' },
  'settings.tutorial': { zh: '使用教程', en: 'Tutorial' },
  'settings.chinese': { zh: '中文', en: '中文' },
  'settings.english': { zh: 'English', en: 'English' },

  // Tutorial
  'tutorial.title': { zh: '使用教程', en: 'User Guide' },
  'tutorial.overview_title': { zh: '概览', en: 'Overview' },
  'tutorial.overview': {
    zh: 'ADBQTools 是一款专为 Android 开发者打造的桌面工具，集成设备管理、实时日志查看（含 Unity 引擎适配）和 APK/AAB 安装功能。',
    en: 'ADBQTools is a desktop utility for Android developers, integrating device management, real-time log viewing (with Unity engine support), and APK/AAB installation.'
  },
  'tutorial.device_title': { zh: '设备管理', en: 'Device Management' },
  'tutorial.device_content': {
    zh: '• 顶部设备栏会自动检测已连接的 Android 设备（USB 和 WiFi）\n• 点击设备芯片可切换当前活动设备\n• WiFi 设备右侧有 × 按钮可断开连接\n• 点击 WiFi 图标打开无线连接对话框\n• 首次连接 WiFi 设备需要先配对（Pair），之后直接连接（Connect）\n• 如遇连接问题，点击刷新按钮重启 ADB 服务',
    en: '• The top device bar auto-detects connected Android devices (USB and WiFi)\n• Click a device chip to switch the active device\n• WiFi devices have an × button to disconnect\n• Click the WiFi icon to open the wireless connection dialog\n• First-time WiFi devices need Pair first, then Connect\n• If connection issues occur, click the refresh button to restart ADB'
  },
  'tutorial.logcat_title': { zh: 'Logcat 日志查看', en: 'Logcat Log Viewer' },
  'tutorial.logcat_content': {
    zh: '• 选择设备后自动开始接收日志\n• 支持结构化查询：tag:Unity, pid:12345, message:error, level:warn\n• 级别过滤：点击 V/D/I/W/E/F 芯片过滤日志级别\n• Unity 模式：点击 U 按钮仅显示 Unity/IL2CPP/Mono 日志\n• 自动换行：点击换行按钮切换长文本折行显示\n• 自动滚动：点击向下箭头按钮自动跟踪最新日志\n• 暂停/继续：暂停日志接收，方便分析\n• 清除：清空当前日志\n• 导出：将日志保存为 .txt 文件\n• 支持正则表达式搜索',
    en: '• Logs start automatically after selecting a device\n• Structured queries: tag:Unity, pid:12345, message:error, level:warn\n• Level filter: click V/D/I/W/E/F chips to filter log levels\n• Unity mode: click U button to show only Unity/IL2CPP/Mono logs\n• Word wrap: toggle line wrapping for long messages\n• Auto-scroll: click the down arrow to auto-follow new logs\n• Pause/Resume: pause log reception for analysis\n• Clear: clear all current logs\n• Export: save logs as a .txt file\n• Regex search supported'
  },
  'tutorial.installer_title': { zh: 'APK/AAB 安装', en: 'APK/AAB Installation' },
  'tutorial.installer_content': {
    zh: '• 拖拽 APK 或 AAB 文件到安装区域，或点击"浏览文件"选择\n• APK 文件直接安装到设备\n• AAB 文件需要签名配置：\n  1. 点击"签名"按钮展开配置面板\n  2. 选择 Keystore 文件（.jks 或 .keystore）\n  3. 输入 Store 密码后自动检测 Key 别名\n  4. 输入 Key 密码\n  5. 点击"安装"开始构建并安装\n• AAB 会自动使用 bundletool 转换为 APK 后安装\n• 安装记录保存在下方的安装历史中',
    en: '• Drag APK or AAB files to the drop zone, or click "Browse Files"\n• APK files install directly to the device\n• AAB files require signing configuration:\n  1. Click "Signing" to expand the config panel\n  2. Select a Keystore file (.jks or .keystore)\n  3. Enter Store password — Key aliases auto-detect\n  4. Enter Key password\n  5. Click "Install" to build and install\n• AAB files are converted to APK via bundletool automatically\n• Install records are saved in the history below'
  },
  'tutorial.wifi_title': { zh: 'WiFi 连接指南', en: 'WiFi Connection Guide' },
  'tutorial.wifi_content': {
    zh: '配对（首次）：\n1. 手机：设置 → 开发者选项 → 无线调试 → 使用配对码配对设备\n2. 记下配对地址和6位配对码\n3. 在 WiFi 对话框选择 Pair，输入配对地址和配对码\n4. 配对成功后可保存 IP 地址\n\n连接（已配对设备）：\n1. 手机：设置 → 开发者选项 → 无线调试\n2. 记下页面顶部显示的 IP 地址和端口\n3. 在 WiFi 对话框选择 Connect，输入连接地址\n4. 注意：连接端口和配对端口不同！',
    en: 'Pairing (first time):\n1. Phone: Settings → Developer Options → Wireless Debugging → Pair device with pairing code\n2. Note the pairing address and 6-digit code\n3. In the WiFi dialog, select Pair, enter the address and code\n4. After pairing, you can save the IP address\n\nConnecting (paired devices):\n1. Phone: Settings → Developer Options → Wireless Debugging\n2. Note the IP address and port shown at the top\n3. In the WiFi dialog, select Connect, enter the address\n4. Note: connection port differs from pairing port!'
  },
};

export function t(key: string, lang: Lang, vars?: Record<string, string>): string {
  let text = translations[key]?.[lang] ?? translations[key]?.['en'] ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      text = text.replace(`{${k}}`, v);
    }
  }
  return text;
}

/** Reactive derived store: returns a function that translates keys */
export const tt = derived(language, ($lang) => {
  return (key: string, vars?: Record<string, string>) => t(key, $lang, vars);
});
