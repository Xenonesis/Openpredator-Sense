import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Activity, Fan, Gauge, Zap, Palette,
  Flame, Cpu, ChevronRight, Sliders,
  Gamepad2,
  Grid, Settings as SettingsIcon, Lock, Unlock,
  Volume2, VolumeX, ArrowDown, ArrowUp,
  RefreshCw, Play, X, ShieldAlert,
  Radio, Minus, Plus, Square, Copy,
} from "lucide-react";
import "./App.css";

export type FanMode = "auto" | "max" | "custom";
export type PerformanceMode = "quiet" | "normal" | "extreme" | "turbo";

export interface FanChannel {
  id: number;
  name: string;
  speed_rpm: number;
  target_percentage: number;
  mode: FanMode;
}

export interface QuickSettings {
  win_key_locked: boolean;
  sticky_keys_enabled: boolean;
  lcd_overdrive: boolean;
  boot_sound: boolean;
  coolboost: boolean;
  kb_brightness: number;
}

export interface NetworkStats {
  download_bytes_per_sec: number;
  upload_bytes_per_sec: number;
}

export interface LightingConfig {
  effect: string;
  speed: number;
  direction: number;
  brightness: number;
  zones: string[];
}

export interface SystemStats {
  cpu_name: string;
  cpu_usage: number;
  cpu_temp: number;
  cpu_frequency_mhz: number;
  cpu_max_frequency_mhz: number;
  gpu_name: string;
  gpu_usage: number;
  gpu_temp: number;
  gpu_clock_mhz: number;
  gpu_max_clock_mhz: number;
  vram_clock_mhz: number;
  ram_used_gb: number;
  ram_total_gb: number;
  current_mode: PerformanceMode;
  fans: {
    cpu: FanChannel;
    gpu: FanChannel;
    coolboost: boolean;
  };
  network: NetworkStats;
  settings: QuickSettings;
  lighting: LightingConfig;
  wmi_ok: boolean;
}

export interface GameProfile {
  id: string;
  name: string;
  exe_name: string;
  path: string;
  performance_mode: PerformanceMode;
  fan_mode: FanMode;
  custom_fan_pct?: number;
  lighting_effect?: string;
  enabled: boolean;
}

export interface AppItem {
  id: string;
  name: string;
  category: string;
  description: string;
  exe_path: string;
  installed: boolean;
  icon_name: string;
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec >= 1024 * 1024) {
    return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
  }
  if (bytesPerSec >= 1024) {
    return `${(bytesPerSec / 1024).toFixed(0)} KB/s`;
  }
  return `${bytesPerSec} B/s`;
}

// ── Authentic Official PredatorSense Mode Icons ─────────────────────────────
function QuietIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 48 48" className={className} fill="none" stroke="currentColor">
      <path d="M16 6 L32 6 L42 16 L42 32 L32 42 L16 42 L6 32 L6 16 Z" strokeWidth="2.5" strokeLinejoin="round" />
      <path d="M21 14 A10.5 10.5 0 1 0 31 29 A12 12 0 0 1 21 14 Z" fill="currentColor" stroke="none" />
      <polygon points="29,15 30.5,18 33,19 30.5,20 29,23 27.5,20 25,19 27.5,18" fill="currentColor" stroke="none" />
      <polygon points="36,23 37,25 39,25.5 37,26 36,28 35,26 33,25.5 35,25" fill="currentColor" stroke="none" />
      <circle cx="28" cy="10" r="1.5" fill="currentColor" stroke="none" />
    </svg>
  );
}

function DefaultIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 48 48" className={className} fill="none" stroke="currentColor">
      <polygon points="12,12 36,12 24,24" strokeWidth="2.5" strokeLinejoin="miter" />
      <polyline points="15,20 24,29 33,20" strokeWidth="2.5" strokeLinecap="square" />
      <polyline points="18,27 24,33 30,27" strokeWidth="2.5" strokeLinecap="square" />
    </svg>
  );
}

function ExtremeIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 48 48" className={className} fill="none" stroke="currentColor">
      <path d="M12 28 A13 13 0 1 1 36 28" strokeWidth="2.8" strokeLinecap="round" strokeDasharray="3 3" />
      <line x1="8" y1="21" x2="14" y2="21" strokeWidth="2.5" strokeLinecap="round" />
      <line x1="6" y1="27" x2="12" y2="27" strokeWidth="2.5" strokeLinecap="round" />
      <line x1="24" y1="28" x2="34" y2="18" strokeWidth="3" strokeLinecap="round" />
      <circle cx="24" cy="28" r="4" fill="currentColor" />
    </svg>
  );
}

function TurboIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 48 48" className={className} fill="none" stroke="currentColor">
      <path d="M12 24 A12 12 0 1 1 36 24" strokeWidth="2" strokeDasharray="4 3" />
      <path d="M24 15 C26 11 32 13 30 17 C28 19 25 21 24 24" strokeWidth="2.5" strokeLinecap="round" />
      <path d="M33 24 C37 26 35 32 31 30 C29 28 27 25 24 24" strokeWidth="2.5" strokeLinecap="round" />
      <path d="M24 33 C22 37 16 35 18 31 C20 29 23 27 24 24" strokeWidth="2.5" strokeLinecap="round" />
      <path d="M15 24 C11 22 13 16 17 18 C19 20 21 23 24 24" strokeWidth="2.5" strokeLinecap="round" />
      <circle cx="24" cy="24" r="3.5" fill="currentColor" />
      <line x1="9" y1="39" x2="39" y2="9" strokeWidth="2" strokeDasharray="3 2" />
    </svg>
  );
}

// ── Exact Speedometer Frequency Gauge from Official PredatorSense ───────────
function FrequencyGauge({
  value,
  max = 5000,
  label,
  subtitle,
  loadingPct,
  isGpuIdle = false,
  color = "#00e5ff",
}: {
  value: number;
  max?: number;
  label: string;
  subtitle: string;
  loadingPct?: number;
  isGpuIdle?: boolean;
  color?: string;
}) {
  const pct = isGpuIdle ? 0 : Math.min(1, Math.max(0, value / max));
  const cx = 110, cy = 110, r = 78;
  const totalAngle = 240;
  const currentAngle = -210 + pct * totalAngle;
  const rad = (currentAngle * Math.PI) / 180;
  const needleX = cx + Math.cos(rad) * (r - 10);
  const needleY = cy + Math.sin(rad) * (r - 10);

  const ticks = Array.from({ length: 17 }, (_, i) => {
    const angle = -210 + (i / 16) * totalAngle;
    const aRad = (angle * Math.PI) / 180;
    const isMajor = i % 2 === 0;
    const innerR = isMajor ? r - 9 : r - 5;
    const outerR = r + 2;
    const x1 = cx + Math.cos(aRad) * innerR;
    const y1 = cy + Math.sin(aRad) * innerR;
    const x2 = cx + Math.cos(aRad) * outerR;
    const y2 = cy + Math.sin(aRad) * outerR;
    const isLit = !isGpuIdle && (i / 16) <= pct;
    return { x1, y1, x2, y2, isMajor, isLit };
  });

  return (
    <div className="flex flex-col items-center select-none">
      <div className="relative w-56 h-48 flex items-center justify-center">
        <svg viewBox="0 0 220 190" className="w-full h-full overflow-visible">
          <defs>
            <linearGradient id={`freqArc-${label}`} x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#082b3d" />
              <stop offset="60%" stopColor={color} />
              <stop offset="100%" stopColor="#00e5ff" />
            </linearGradient>
            <filter id={`bloomFreq-${label}`} x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="3" result="blur" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
          </defs>

          {/* Tick Marks */}
          {ticks.map((t, idx) => (
            <line
              key={idx}
              x1={t.x1}
              y1={t.y1}
              x2={t.x2}
              y2={t.y2}
              stroke={t.isLit ? color : "#152438"}
              strokeWidth={t.isMajor ? 2.5 : 1}
              strokeLinecap="round"
              filter={t.isLit ? `url(#bloomFreq-${label})` : undefined}
            />
          ))}

          {/* Outer Arc Track */}
          <circle
            cx={cx}
            cy={cy}
            r={r - 18}
            fill="none"
            stroke="#0b1320"
            strokeWidth="7"
            strokeDasharray={`${(totalAngle / 360) * (2 * Math.PI * (r - 18))} ${2 * Math.PI * (r - 18)}`}
            className="-rotate-210 origin-center"
          />

          {/* Active Lit Arc */}
          {!isGpuIdle && (
            <circle
              cx={cx}
              cy={cy}
              r={r - 18}
              fill="none"
              stroke={`url(#freqArc-${label})`}
              strokeWidth="7"
              strokeDasharray={`${pct * ((totalAngle / 360) * (2 * Math.PI * (r - 18)))} ${2 * Math.PI * (r - 18)}`}
              strokeLinecap="round"
              className="-rotate-210 origin-center transition-all duration-700 ease-out"
              filter={`url(#bloomFreq-${label})`}
            />
          )}

          {/* Needle */}
          <line
            x1={cx}
            y1={cy}
            x2={needleX}
            y2={needleY}
            stroke={isGpuIdle ? "#1c2b42" : color}
            strokeWidth="3"
            strokeLinecap="round"
            filter={!isGpuIdle ? `url(#bloomFreq-${label})` : undefined}
            className="transition-all duration-700 ease-out"
          />
          <circle cx={cx} cy={cy} r="6" fill="#070c15" stroke={isGpuIdle ? "#1c2b42" : color} strokeWidth="2.5" />
        </svg>

        {/* Center Readout & Optional (i) Info icon */}
        <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none pt-2">
          {isGpuIdle ? (
            <div className="flex flex-col items-center pointer-events-auto">
              <div
                title="GPU is currently in power-saving idle state (NVIDIA Optimus). Run a 3D application or game to activate discrete graphics."
                className="w-4 h-4 rounded-full bg-slate-800 hover:bg-cyan-950 border border-slate-600 hover:border-cyan-400 text-[10px] font-mono font-bold flex items-center justify-center text-slate-300 hover:text-cyan-300 cursor-help mb-1 transition-all"
              >
                i
              </div>
              <span className="text-3xl font-black font-mono tracking-tight text-slate-400">
                --
              </span>
              <span className="text-[11px] font-mono font-semibold tracking-wider text-slate-500 uppercase">
                MHz
              </span>
            </div>
          ) : (
            <>
              <span className="text-3xl font-black font-mono tracking-tight text-white drop-shadow-[0_0_10px_rgba(0,229,255,0.4)]">
                {value}
              </span>
              <span className="text-[11px] font-mono font-semibold tracking-wider text-slate-400 uppercase">
                MHz
              </span>
            </>
          )}
        </div>

        {/* ◆ MAX Marker */}
        <div className="absolute right-0 bottom-4 text-[10px] font-mono font-semibold text-cyan-400 flex items-center gap-0.5">
          <span>◆</span>
          <span>MAX</span>
        </div>
      </div>

      {/* Bottom Labels exactly matching Image #1 */}
      <div className="text-center mt-1 space-y-0.5">
        <p className="text-sm font-bold font-mono tracking-widest uppercase text-slate-200">
          {label}
        </p>
        <p className="text-xs text-slate-400 font-sans">
          {subtitle}
        </p>
        <p className="text-[11px] font-mono text-slate-500">
          Loading {isGpuIdle ? "--%" : `${Math.round(loadingPct ?? 0)}%`}
        </p>
      </div>
    </div>
  );
}

// ── AeroBlade 3D Calibrated Tachometer ─────────────────────────────────────────
function AeroBladeTachometer({
  rpm,
  maxRpm = 7500,
  label,
  channel,
  temp,
  mode,
  theme = "cyan",
}: {
  rpm: number;
  maxRpm?: number;
  label: string;
  channel: string;
  temp: number;
  mode: FanMode;
  theme?: "cyan" | "purple";
}) {
  const pct = Math.min(1, Math.max(0, rpm / maxRpm));
  const r = 68, cx = 90, cy = 90;
  // Sweep from -210 deg to +30 deg (240 deg arc)
  const totalAngle = 240;
  const currentAngle = -210 + pct * totalAngle;
  const rad = (currentAngle * Math.PI) / 180;
  const needleX = cx + Math.cos(rad) * (r - 12);
  const needleY = cy + Math.sin(rad) * (r - 12);

  const primaryColor = theme === "cyan" ? "#00e5ff" : "#b026ff";
  const glowColor = theme === "cyan" ? "rgba(0,229,255,0.4)" : "rgba(176,38,255,0.4)";
  const spinSpeed = rpm > 100 ? `${Math.max(0.15, 2600 / rpm).toFixed(2)}s` : "4s";

  // Generate 13 calibrated tick marks
  const ticks = Array.from({ length: 13 }, (_, i) => {
    const angle = -210 + (i / 12) * totalAngle;
    const aRad = (angle * Math.PI) / 180;
    const isMajor = i % 2 === 0;
    const innerR = isMajor ? r - 8 : r - 4;
    const outerR = r + 2;
    const x1 = cx + Math.cos(aRad) * innerR;
    const y1 = cy + Math.sin(aRad) * innerR;
    const x2 = cx + Math.cos(aRad) * outerR;
    const y2 = cy + Math.sin(aRad) * outerR;
    const isLit = (i / 12) <= pct;
    return { x1, y1, x2, y2, isMajor, isLit, rpmVal: i * (maxRpm / 12) };
  });

  return (
    <div className="relative p-5 rounded-xl bg-gradient-to-b from-[#0c121d] via-[#090d16] to-[#06090f] border border-[#172338] shadow-2xl flex flex-col items-center justify-between overflow-hidden">
      {/* Background corner graticule decoration */}
      <div className="absolute top-2 left-2 flex items-center gap-1.5 opacity-60">
        <span className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: primaryColor }} />
        <span className="text-[10px] font-mono tracking-widest uppercase text-slate-400">
          AEROBLADE 3D // {channel}
        </span>
      </div>

      <div className="absolute top-2 right-2 flex items-center gap-2">
        <span className="text-[10px] font-mono font-semibold px-2 py-0.5 rounded bg-[#101726] border border-[#1b273d] text-slate-300 uppercase">
          {mode === "max" ? "OVERDRIVE 100%" : mode === "custom" ? "CALIBRATED" : "AUTO-CURVE"}
        </span>
      </div>

      {/* Main Tachometer Dial */}
      <div className="relative w-44 h-44 mt-4 flex items-center justify-center">
        <svg viewBox="0 0 180 180" className="w-full h-full">
          <defs>
            <linearGradient id={`sweepGrad-${theme}`} x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#1e293b" />
              <stop offset="60%" stopColor={primaryColor} />
              <stop offset="100%" stopColor="#ff1a40" />
            </linearGradient>
            <filter id={`bloom-${theme}`} x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="3" result="blur" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
          </defs>

          {/* Calibrated Ticks */}
          {ticks.map((t, idx) => (
            <line
              key={idx}
              x1={t.x1}
              y1={t.y1}
              x2={t.x2}
              y2={t.y2}
              stroke={t.isLit ? primaryColor : "#243248"}
              strokeWidth={t.isMajor ? 2.5 : 1}
              strokeLinecap="round"
              filter={t.isLit ? `url(#bloom-${theme})` : undefined}
            />
          ))}

          {/* Dial Base Track */}
          <circle
            cx={cx}
            cy={cy}
            r={r - 18}
            fill="none"
            stroke="#0e1624"
            strokeWidth="6"
          />

          {/* Active Value Sweep Ring */}
          <circle
            cx={cx}
            cy={cy}
            r={r - 18}
            fill="none"
            stroke={`url(#sweepGrad-${theme})`}
            strokeWidth="6"
            strokeDasharray={`${pct * (2 * Math.PI * (r - 18))} ${2 * Math.PI * (r - 18)}`}
            strokeLinecap="round"
            className="-rotate-90 origin-center transition-all duration-700 ease-out"
            filter={`url(#bloom-${theme})`}
          />

          {/* Needle Indicator */}
          <line
            x1={cx}
            y1={cy}
            x2={needleX}
            y2={needleY}
            stroke={primaryColor}
            strokeWidth="2.5"
            strokeLinecap="round"
            filter={`url(#bloom-${theme})`}
            className="transition-all duration-700 ease-out"
          />
          <circle cx={cx} cy={cy} r="5" fill="#0f172a" stroke={primaryColor} strokeWidth="2.5" />
        </svg>

        {/* Central Turbine & RPM Digital Display */}
        <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none pt-4">
          <Fan
            className={`w-6 h-6 mb-1 ${rpm > 100 ? "animate-spin" : ""}`}
            style={{
              color: primaryColor,
              animationDuration: spinSpeed,
              filter: `drop-shadow(0 0 6px ${glowColor})`,
            }}
          />
          <span className="text-2xl font-black font-mono tracking-tighter" style={{ color: primaryColor }}>
            {rpm > 0 ? rpm.toLocaleString() : "0"}
          </span>
          <span className="text-[10px] font-mono tracking-widest uppercase text-slate-500">
            RPM TACH
          </span>
        </div>
      </div>

      {/* Sensor Footer */}
      <div className="w-full mt-3 pt-3 border-t border-[#131b2a] flex items-center justify-between text-xs font-mono">
        <span className="text-slate-400 flex items-center gap-1.5 font-semibold">
          <ThermometerIcon className="w-3.5 h-3.5" style={{ color: primaryColor }} />
          <span>{label} CORE</span>
        </span>
        <span className="font-bold text-slate-200">
          {temp > 0 ? `${temp.toFixed(0)}°C` : "—"}
        </span>
      </div>
    </div>
  );
}

function ThermometerIcon({ className, style }: { className?: string; style?: React.CSSProperties }) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className} style={style}>
      <path d="M14 4v10.54a4 4 0 1 1-4 0V4a2 2 0 0 1 4 0Z" />
    </svg>
  );
}

// ── Stat Bar ─────────────────────────────────────────────────────────────────
function TacticalStatCard({
  title,
  subtitle,
  value,
  max,
  unit,
  color,
  extra,
}: {
  title: string;
  subtitle: string;
  value: number;
  max: number;
  unit: string;
  color: string;
  extra?: string;
}) {
  const pct = Math.min(100, (value / max) * 100);
  return (
    <div className="p-4 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#070b12] border border-[#172338] shadow-lg space-y-3">
      <div className="flex justify-between items-start">
        <div>
          <p className="text-[11px] font-mono font-semibold uppercase tracking-wider text-slate-400">{title}</p>
          <p className="text-xs text-slate-500 truncate max-w-[140px]">{subtitle}</p>
        </div>
        <span className="text-xl font-black font-mono" style={{ color }}>
          {value.toFixed(1)}<span className="text-xs font-normal ml-0.5">{unit}</span>
        </span>
      </div>

      <div className="h-1.5 rounded-full bg-[#101726] border border-[#1b2638] overflow-hidden">
        <div
          className="h-full rounded-full transition-all duration-700 ease-out"
          style={{ width: `${pct}%`, backgroundColor: color, boxShadow: `0 0 8px ${color}88` }}
        />
      </div>

      {extra && (
        <div className="text-[10px] font-mono text-slate-500 flex justify-between pt-1 border-t border-[#121927]">
          <span>TELEMETRY</span>
          <span className="text-slate-300 font-semibold">{extra}</span>
        </div>
      )}
    </div>
  );
}

// ── Main Application Component ────────────────────────────────────────────────
export default function App() {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"home" | "fans" | "lighting" | "monitoring" | "gamesync" | "appcenter">("home");
  const [showSettings, setShowSettings] = useState(false);
  const [showModesIntro, setShowModesIntro] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    invoke<boolean>("window_is_maximized")
      .then(setIsMaximized)
      .catch(console.error);

    const onResize = () => {
      invoke<boolean>("window_is_maximized")
        .then(setIsMaximized)
        .catch(console.error);
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  const handleMinimize = () => {
    invoke("window_minimize").catch(console.error);
  };

  const handleToggleMaximize = async () => {
    await invoke("window_toggle_maximize").catch(console.error);
    const max = await invoke<boolean>("window_is_maximized").catch(() => false);
    setIsMaximized(max);
  };

  const handleClose = () => {
    invoke("window_close").catch(console.error);
  };

  // Fan speeds
  const [cpuSpeed, setCpuSpeed] = useState(60);
  const [gpuSpeed, setGpuSpeed] = useState(60);

  // RGB States
  const [zones, setZones] = useState(["#00AEC7", "#00AEC7", "#00AEC7", "#00AEC7"]);
  const [activeZone, setActiveZone] = useState<number>(0);
  const [selectedEffect, setSelectedEffect] = useState<string>("static");
  const [effectSpeed, setEffectSpeed] = useState<number>(2);
  const [effectDirection, setEffectDirection] = useState<number>(1);
  const [kbBrightness, setKbBrightness] = useState<number>(100);

  // Game Sync state
  const [games, setGames] = useState<GameProfile[]>([]);
  const [scanningGames, setScanningGames] = useState(false);
  const [newGameName, setNewGameName] = useState("");
  const [newGameExe, setNewGameExe] = useState("");
  const [showAddGame, setShowAddGame] = useState(false);

  // App Center state
  const [apps, setApps] = useState<AppItem[]>([]);

  // Telemetry stream history
  const [netHistory, setNetHistory] = useState<{ down: number[]; up: number[] }>({ down: [], up: [] });

  const pendingRef = useRef(false);

  const fetchStats = async () => {
    if (pendingRef.current) return;
    pendingRef.current = true;
    try {
      const data = await invoke<SystemStats>("get_system_stats");
      setStats(data);
      setError(null);

      if (data.lighting?.zones?.length === 4) {
        setZones(data.lighting.zones);
      }

      setNetHistory((prev) => {
        const nextDown = [...prev.down, data.network.download_bytes_per_sec].slice(-24);
        const nextUp = [...prev.up, data.network.upload_bytes_per_sec].slice(-24);
        return { down: nextDown, up: nextUp };
      });
    } catch (e) {
      setError(String(e));
    } finally {
      pendingRef.current = false;
    }
  };

  useEffect(() => {
    fetchStats();
    const id = setInterval(fetchStats, 1500);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    invoke<GameProfile[]>("scan_installed_games")
      .then(setGames)
      .catch(console.error);

    invoke<AppItem[]>("get_app_center_items")
      .then(setApps)
      .catch(console.error);
  }, []);

  // ── Hardware Trigger Actions ────────────────────────────────────────────────
  const setMode = (mode: PerformanceMode) => {
    invoke("set_performance_mode", { mode }).catch(console.error);
    if (stats) setStats({ ...stats, current_mode: mode });
  };

  const setFanMode = (fanId: number, mode: FanMode, pct?: number) => {
    invoke("set_fan_mode", { fanId, mode, customPct: pct ?? null }).catch(console.error);
    if (stats) {
      const isCpu = fanId === 1;
      setStats({
        ...stats,
        fans: {
          ...stats.fans,
          cpu: isCpu ? { ...stats.fans.cpu, mode, target_percentage: pct ?? stats.fans.cpu.target_percentage } : stats.fans.cpu,
          gpu: !isCpu ? { ...stats.fans.gpu, mode, target_percentage: pct ?? stats.fans.gpu.target_percentage } : stats.fans.gpu,
        },
      });
    }
  };

  const toggleCoolboost = () => {
    const next = !(stats?.fans.coolboost ?? false);
    invoke("set_coolboost", { enabled: next }).catch(console.error);
    if (stats) setStats({ ...stats, fans: { ...stats.fans, coolboost: next } });
  };

  const toggleWinKeyLock = () => {
    const next = !(stats?.settings.win_key_locked ?? false);
    invoke("set_win_key_lock", { locked: next }).catch(console.error);
    if (stats) setStats({ ...stats, settings: { ...stats.settings, win_key_locked: next } });
  };

  const toggleStickyKeys = () => {
    const next = !(stats?.settings.sticky_keys_enabled ?? false);
    invoke("set_sticky_keys", { enabled: next }).catch(console.error);
    if (stats) setStats({ ...stats, settings: { ...stats.settings, sticky_keys_enabled: next } });
  };

  const toggleLcdOverdrive = () => {
    const next = !(stats?.settings.lcd_overdrive ?? false);
    invoke("set_lcd_overdrive", { enabled: next }).catch(console.error);
    if (stats) setStats({ ...stats, settings: { ...stats.settings, lcd_overdrive: next } });
  };

  const toggleBootSound = () => {
    const next = !(stats?.settings.boot_sound ?? true);
    invoke("set_boot_sound", { enabled: next }).catch(console.error);
    if (stats) setStats({ ...stats, settings: { ...stats.settings, boot_sound: next } });
  };

  const handleBrightnessChange = (val: number) => {
    setKbBrightness(val);
    invoke("set_kb_brightness", { pct: val }).catch(console.error);
  };

  const applyRgb = (idx: number, hex: string) => {
    const nz = [...zones];
    nz[idx] = hex;
    setZones(nz);
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    invoke("set_rgb_zone", { zone: idx + 1, r, g, b }).catch(console.error);
  };

  const applyRgbToAll = (hex: string) => {
    setZones([hex, hex, hex, hex]);
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    [1, 2, 3, 4].forEach((zone) => {
      invoke("set_rgb_zone", { zone, r, g, b }).catch(console.error);
    });
  };

  const applyDynamicEffect = (
    eff: string,
    spd: number = effectSpeed,
    dir: number = effectDirection,
    bri: number = kbBrightness,
  ) => {
    setSelectedEffect(eff);
    const effId = matchEffectId(eff);
    const primary = zones[0] || "#00AEC7";
    const r = parseInt(primary.slice(1, 3), 16);
    const g = parseInt(primary.slice(3, 5), 16);
    const b = parseInt(primary.slice(5, 7), 16);
    invoke("set_lighting_effect", {
      effect: effId,
      speed: spd,
      direction: dir,
      brightness: bri,
      r,
      g,
      b,
    }).catch(console.error);
  };

  const matchEffectId = (name: string): number => {
    switch (name) {
      case "static": return 1;
      case "breathing": return 2;
      case "wave": return 3;
      case "zoom": return 4;
      case "shift": return 5;
      case "neon": return 10;
      case "rainbow": return 12;
      default: return 1;
    }
  };

  const applyPresetProfile = (preset: { name: string; zones: string[]; effect: string }) => {
    setZones(preset.zones);
    setSelectedEffect(preset.effect);
    preset.zones.forEach((c, i) => applyRgb(i, c));
    if (preset.effect !== "static") {
      applyDynamicEffect(preset.effect);
    }
  };

  const rescanGames = async () => {
    setScanningGames(true);
    try {
      const scanned = await invoke<GameProfile[]>("scan_installed_games");
      setGames(scanned);
    } finally {
      setScanningGames(false);
    }
  };

  const addCustomGame = () => {
    if (!newGameName.trim() || !newGameExe.trim()) return;
    const g: GameProfile = {
      id: `custom_${Date.now()}`,
      name: newGameName.trim(),
      exe_name: newGameExe.trim(),
      path: "",
      performance_mode: "turbo",
      fan_mode: "max",
      custom_fan_pct: 100,
      lighting_effect: "wave",
      enabled: true,
    };
    setGames([g, ...games]);
    setNewGameName("");
    setNewGameExe("");
    setShowAddGame(false);
  };

  const launchAppItem = (path: string) => {
    invoke("launch_app", { path }).catch((err) => alert(String(err)));
  };

  const mode = stats?.current_mode ?? "normal";
  const wmiOk = stats?.wmi_ok ?? false;

  const TABS = [
    { id: "home",       label: "COMMAND DECK", icon: Gauge,      sub: "Primary Telemetry" },
    { id: "fans",       label: "AEROBLADE 3D", icon: Fan,        sub: "Dual Fan Curves" },
    { id: "lighting",   label: "PULSAR RGB",   icon: Palette,    sub: "4-Zone Illumination" },
    { id: "monitoring", label: "DIAGNOSTICS",  icon: Activity,   sub: "System Oscilloscope" },
    { id: "gamesync",   label: "GAME SYNC",    icon: Gamepad2,   sub: "Auto Flight Profiles" },
    { id: "appcenter",  label: "LAUNCHPAD",    icon: Grid,       sub: "OEM & Utility Hub" },
  ] as const;

  const PRESETS = [
    { name: "Predator Cyan", zones: ["#00AEC7", "#00AEC7", "#00AEC7", "#00AEC7"], effect: "static" },
    { name: "Cyberpunk Neon", zones: ["#00E5FF", "#E0007B", "#7B00FF", "#00E5FF"], effect: "neon" },
    { name: "Inferno Crimson", zones: ["#FF0033", "#FF3300", "#FF6600", "#FF0033"], effect: "breathing" },
    { name: "Toxic Matrix", zones: ["#00FF66", "#33FF00", "#66FF33", "#00FF66"], effect: "wave" },
    { name: "Stealth Dark", zones: ["#111827", "#111827", "#111827", "#111827"], effect: "static" },
  ];

  return (
    <div className="flex h-screen bg-[#05070a] text-slate-100 font-sans select-none overflow-hidden bg-cockpit-grid">

      {/* ── Precision Avionics Sidebar ── */}
      <aside className="w-56 lg:w-64 bg-[#090d15] border-r border-[#152033] flex flex-col justify-between p-3.5 z-20 shadow-2xl shrink-0">
        <div>
          {/* Predator Brand Shield */}
          <div className="px-3 py-3.5 mb-5 border-b border-[#152033] flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-[#00e5ff] to-[#0066ff] flex items-center justify-center shadow-lg shadow-cyan-500/25 border border-cyan-400/30">
              <Zap className="w-5 h-5 text-black fill-current animate-pulse" />
            </div>
            <div>
              <div className="flex items-center gap-1.5">
                <span className="text-base font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 via-teal-200 to-blue-400 font-mono">
                  PREDATOR
                </span>
                <span className="text-[9px] px-1 py-0.2 rounded bg-cyan-950 text-cyan-400 font-mono border border-cyan-800/40">
                  PRO
                </span>
              </div>
              <p className="text-[9px] font-mono tracking-widest uppercase text-slate-500">
                HELIOS 300 // PH315-55
              </p>
            </div>
          </div>

          {/* Nav Deck */}
          <nav className="space-y-1">
            {TABS.map(({ id, label, icon: Icon, sub }) => {
              const active = activeTab === id;
              return (
                <button
                  key={id}
                  onClick={() => setActiveTab(id)}
                  className={`w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-left transition-all duration-150 border ${
                    active
                      ? "bg-gradient-to-r from-[#0e1726] to-[#0b121e] text-cyan-300 border-[#00e5ff]/50 shadow-lg shadow-cyan-950/50"
                      : "text-slate-400 hover:text-slate-200 hover:bg-[#0c121e] border-transparent"
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-7 h-7 rounded flex items-center justify-center ${
                        active ? "bg-cyan-950/80 text-cyan-400 border border-cyan-500/40" : "bg-[#0c121c] text-slate-500"
                      }`}
                    >
                      <Icon className="w-4 h-4" />
                    </div>
                    <div>
                      <p className={`text-xs font-bold font-mono tracking-wider ${active ? "text-cyan-300" : "text-slate-300"}`}>
                        {label}
                      </p>
                      <p className="text-[9px] font-mono text-slate-500 uppercase">{sub}</p>
                    </div>
                  </div>
                  {active && <ChevronRight className="w-3.5 h-3.5 text-cyan-400" />}
                </button>
              );
            })}
          </nav>
        </div>

        {/* Bottom Hardware Console Bar */}
        <div className="space-y-2.5">
          <button
            onClick={() => setShowSettings(true)}
            className="w-full flex items-center justify-between p-2 rounded-lg bg-[#0c121c] border border-[#162134] text-xs font-semibold text-slate-300 hover:text-cyan-300 hover:border-cyan-500/50 transition-all font-mono"
          >
            <span className="flex items-center gap-2 text-[11px]">
              <SettingsIcon className="w-3.5 h-3.5 text-cyan-400" /> SYSTEM ARCHITECTURE
            </span>
            <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-[#10192a] text-cyan-400 border border-cyan-800/40">
              BIOS
            </span>
          </button>

          {/* ACPI Link State */}
          <div className="p-2.5 rounded-lg bg-[#0a0f19] border border-[#152033] flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span
                className={`w-2 h-2 rounded-full ${
                  wmiOk ? "bg-emerald-400 shadow-[0_0_8px_#34d399]" : "bg-amber-400 shadow-[0_0_8px_#fbbf24]"
                }`}
              />
              <span className="text-[10px] font-mono font-semibold tracking-wider text-slate-300">
                {wmiOk ? "ACPI WMI LINK ONLINE" : "RESTRICTED ACCESS"}
              </span>
            </div>
            <span className="text-[9px] font-mono text-cyan-400 font-bold">1.5s TICK</span>
          </div>
        </div>
      </aside>

      {/* ── Main Cockpit Canvas ── */}
      <main className="flex-1 flex flex-col bg-[#05070a] overflow-y-auto">

        {/* ── Avionics Flight Header ── */}
        {/* ── Avionics Flight Header with Native Drag Region & Window Controls ── */}
        <header
          data-tauri-drag-region
          onDoubleClick={handleToggleMaximize}
          className="h-16 px-4 md:px-6 border-b border-[#152033] flex items-center justify-between bg-[#080c14]/95 backdrop-blur sticky top-0 z-30 shadow-lg select-none"
        >
          {/* Flight Mode Switcher */}
          <div className="flex items-center gap-2 md:gap-3" data-tauri-drag-region="false">
            <span className="text-[10px] font-mono tracking-widest uppercase text-slate-500 font-bold hidden xl:inline">MODE SELECT:</span>
            <div className="flex bg-[#0a0f19] p-1 rounded-lg border border-[#182337] gap-1 shadow-inner">
              {(["quiet", "normal", "extreme", "turbo"] as PerformanceMode[]).map((m) => {
                const isCurrent = mode === m;
                return (
                  <button
                    key={m}
                    onClick={() => setMode(m)}
                    className={`px-2.5 md:px-3 py-1 text-[10px] font-bold uppercase rounded font-mono tracking-widest transition-all relative cursor-pointer ${
                      isCurrent
                        ? m === "turbo"
                          ? "bg-gradient-to-r from-red-600 to-rose-600 text-white shadow-lg shadow-red-600/30 font-black animate-pulse"
                          : "bg-cyan-500 text-black shadow-md shadow-cyan-500/30 font-black"
                        : "text-slate-400 hover:text-slate-200 hover:bg-[#121a2a]"
                    }`}
                  >
                    {m}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Machined Cockpit Switches */}
          <div className="hidden sm:flex items-center gap-1.5 md:gap-2" data-tauri-drag-region="false">
            {/* Win Key Lock */}
            <button
              onClick={toggleWinKeyLock}
              title={stats?.settings.win_key_locked ? "Windows Key is BLOCKED" : "Lock Windows Key for Gaming"}
              className={`flex items-center gap-1.5 px-2 md:px-2.5 py-1 rounded text-[11px] font-mono font-semibold transition-all border cursor-pointer ${
                stats?.settings.win_key_locked
                  ? "bg-cyan-950/80 text-cyan-300 border-cyan-400 shadow-[0_0_10px_rgba(0,229,255,0.25)]"
                  : "bg-[#0b101b] text-slate-400 border-[#192439] hover:text-slate-200"
              }`}
            >
              {stats?.settings.win_key_locked ? <Lock className="w-3.5 h-3.5 text-cyan-400" /> : <Unlock className="w-3.5 h-3.5 text-slate-500" />}
              <span className="hidden md:inline">WIN LOCK</span>
            </button>

            {/* LCD 3ms Overdrive */}
            <button
              onClick={toggleLcdOverdrive}
              title="Engage 3ms LCD Response Overdrive"
              className={`flex items-center gap-1.5 px-2 md:px-2.5 py-1 rounded text-[11px] font-mono font-semibold transition-all border cursor-pointer ${
                stats?.settings.lcd_overdrive
                  ? "bg-cyan-950/80 text-cyan-300 border-cyan-400 shadow-[0_0_10px_rgba(0,229,255,0.25)]"
                  : "bg-[#0b101b] text-slate-400 border-[#192439] hover:text-slate-200"
              }`}
            >
              <Radio className="w-3 h-3 text-cyan-400" />
              <span className="hidden md:inline">LCD 3ms</span>
            </button>

            {/* Boot Chime */}
            <button
              onClick={toggleBootSound}
              title="Toggle BIOS Boot Roar Sound"
              className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] font-mono transition-all border cursor-pointer ${
                stats?.settings.boot_sound
                  ? "bg-[#0f192b] text-cyan-300 border-cyan-500/50"
                  : "bg-[#0b101b] text-slate-500 border-[#192439]"
              }`}
            >
              {stats?.settings.boot_sound ? <Volume2 className="w-3.5 h-3.5 text-cyan-400" /> : <VolumeX className="w-3.5 h-3.5 text-slate-600" />}
            </button>
          </div>

          {/* Right Section: Telemetry + Window Controls */}
          <div className="flex items-center gap-2 md:gap-3" data-tauri-drag-region="false">
            {/* Telemetry Sensor Pill */}
            <div className="hidden lg:flex items-center gap-3 text-xs font-mono bg-[#0b101a] px-3 py-1.5 rounded-lg border border-[#162236]">
              <span className="flex items-center gap-1 text-slate-300">
                <Cpu className="w-3.5 h-3.5 text-cyan-400" />
                <strong className="text-cyan-300">{Math.round(stats?.cpu_temp ?? 0)}°C</strong>
              </span>
              <span className="text-slate-600">|</span>
              <span className="flex items-center gap-1 text-slate-300">
                <Flame className="w-3.5 h-3.5 text-orange-400" />
                <strong className="text-orange-400">{Math.round(stats?.gpu_temp ?? 0)}°C</strong>
              </span>
              <span className="text-slate-600">|</span>
              <span className="flex items-center gap-1 text-emerald-400">
                <ArrowDown className="w-3 h-3" />
                <strong>{formatSpeed(stats?.network.download_bytes_per_sec ?? 0)}</strong>
              </span>
            </div>

            {/* Window Controls (Minimize, Maximize/Restore, Close) */}
            <div className="flex items-center gap-1 pl-2 md:pl-3 border-l border-[#19263c]">
              <button
                onClick={handleMinimize}
                title="Minimize Window"
                className="w-7 h-7 rounded hover:bg-[#152033] text-slate-400 hover:text-white flex items-center justify-center transition-all cursor-pointer"
              >
                <Minus className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={handleToggleMaximize}
                title={isMaximized ? "Restore Window" : "Maximize Window"}
                className="w-7 h-7 rounded hover:bg-[#152033] text-slate-400 hover:text-cyan-300 flex items-center justify-center transition-all cursor-pointer"
              >
                {isMaximized ? (
                  <Copy className="w-3 h-3 rotate-180" />
                ) : (
                  <Square className="w-3 h-3" />
                )}
              </button>
              <button
                onClick={handleClose}
                title="Close Application"
                className="w-7 h-7 rounded hover:bg-red-600 text-slate-400 hover:text-white flex items-center justify-center transition-all cursor-pointer"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </header>

        {error && (
          <div className="mx-6 mt-4 p-3 rounded-lg bg-red-950/40 border border-red-500/40 text-xs font-mono text-red-300 flex items-center gap-2">
            <ShieldAlert className="w-4 h-4 text-red-400 shrink-0" />
            <span>HARDWARE BUS ALERT: {error}</span>
          </div>
        )}

        {/* ── Content Viewport ── */}
        <div className="p-6 space-y-6 flex-1">

          {/* ══════════════════════════════════════════════════════════════════
              TAB: COMMAND DECK (HOME)
             ══════════════════════════════════════════════════════════════════ */}
          {activeTab === "home" && stats && (
            <div className="space-y-6">
              {/* Turbo Warning Alert */}
              {mode === "turbo" && (
                <div className="p-4 rounded-xl bg-gradient-to-r from-red-950/70 via-rose-950/30 to-[#070b12] border border-red-500/50 flex items-center justify-between shadow-xl">
                  <div className="flex items-center gap-3.5">
                    <div className="w-9 h-9 rounded-lg bg-red-600/20 border border-red-500/50 flex items-center justify-center">
                      <Flame className="w-6 h-6 text-red-500 animate-bounce" />
                    </div>
                    <div>
                      <p className="text-sm font-black text-red-400 font-mono tracking-wider uppercase">
                        TURBO OVERDRIVE ENGAGED // MAXIMUM POWER LIMITS
                      </p>
                      <p className="text-xs text-slate-400 font-mono">
                        Hardware fans pinned to maximum physical duty cycle. Dynamic GPU core boost active.
                      </p>
                    </div>
                  </div>
                  <span className="px-3 py-1 rounded bg-red-600 text-white font-mono text-[11px] font-black tracking-widest uppercase shadow-md shadow-red-600/40">
                    7,100+ RPM ROAR
                  </span>
                </div>
              )}
              {/* ── Authentic Official PredatorSense Mode & Frequency Command Deck (Image #1) ── */}
              <div className="p-6 rounded-2xl bg-gradient-to-b from-[#0c121d] via-[#090d16] to-[#06090f] border border-[#172338] shadow-2xl relative overflow-hidden">
                {/* Header with Title and Introduction of Modes link */}
                <div className="flex items-center justify-between mb-4">
                  <div className="w-1/3" />
                  <h2 className="text-base font-bold font-mono tracking-widest text-slate-300 text-center uppercase">
                    Mode
                  </h2>
                  <div className="w-1/3 flex justify-end">
                    <button
                      onClick={() => setShowModesIntro(true)}
                      className="text-xs font-mono text-slate-400 hover:text-cyan-400 underline decoration-slate-600 hover:decoration-cyan-400 transition-all cursor-pointer flex items-center gap-1"
                    >
                      <span>Introduction of Modes</span>
                    </button>
                  </div>
                </div>

                {/* 4 Mode Buttons with Exact Icons from Image #1 */}
                <div className="flex justify-center gap-3 sm:gap-4 mb-6">
                  {[
                    { id: "quiet", label: "Quiet", icon: QuietIcon, color: "#cbd5e1" },
                    { id: "normal", label: "Default", icon: DefaultIcon, color: "#00e5ff" },
                    { id: "extreme", label: "Extreme", icon: ExtremeIcon, color: "#ff8800" },
                    { id: "turbo", label: "Turbo", icon: TurboIcon, color: "#ff1100" },
                  ].map(({ id, label, icon: Icon, color }) => {
                    const isCurrent = mode === id;
                    return (
                      <button
                        key={id}
                        onClick={() => {
                          setMode(id as PerformanceMode);
                          const pulseHex = id === "quiet" ? "#FFFFFF" : id === "normal" ? "#00AEC7" : id === "extreme" ? "#FF8800" : "#FF1100";
                          applyRgbToAll(pulseHex);
                        }}
                        style={{
                          borderColor: isCurrent ? color : "#182438",
                          boxShadow: isCurrent ? `0 0 16px ${color}44` : undefined,
                        }}
                        className={`w-24 sm:w-28 h-20 rounded-xl border-2 flex flex-col items-center justify-center gap-1.5 transition-all cursor-pointer ${
                          isCurrent
                            ? "bg-[#0c1524] text-white"
                            : "bg-[#090d16] text-slate-500 hover:text-slate-300 hover:bg-[#0e1422]"
                        }`}
                      >
                        <Icon className="w-7 h-7" />
                        <span
                          className={`text-xs font-mono font-bold tracking-wider ${
                            isCurrent ? "text-slate-100" : "text-slate-500"
                          }`}
                          style={isCurrent ? { color } : undefined}
                        >
                          {label}
                        </span>
                      </button>
                    );
                  })}
                </div>

                {/* Dual Frequency Speedometer Gauges from Image #1 */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-[#131d2e]">
                  {/* CPU Frequency Gauge */}
                  <FrequencyGauge
                    value={stats.cpu_frequency_mhz > 0 ? stats.cpu_frequency_mhz : 3464}
                    max={stats.cpu_max_frequency_mhz > 0 ? stats.cpu_max_frequency_mhz : 5000}
                    label="CPU"
                    subtitle="Frequency"
                    loadingPct={stats.cpu_usage}
                    color="#00e5ff"
                  />

                  {/* GPU Core Clock Gauge */}
                  <FrequencyGauge
                    value={stats.gpu_clock_mhz}
                    max={stats.gpu_max_clock_mhz > 0 ? stats.gpu_max_clock_mhz : 1850}
                    label="GPU"
                    subtitle="Core Clock"
                    loadingPct={stats.gpu_usage}
                    isGpuIdle={stats.gpu_clock_mhz === 0}
                    color="#00e5ff"
                  />
                </div>
              </div>

              {/* Dual AeroBlade Tachometers */}
              <div className="grid grid-cols-1 xl:grid-cols-2 gap-4 lg:gap-6">
                <AeroBladeTachometer
                  rpm={stats.fans.cpu.speed_rpm}
                  label="CPU"
                  channel="FAN 01"
                  temp={stats.cpu_temp}
                  mode={stats.fans.cpu.mode}
                  theme="cyan"
                />
                <AeroBladeTachometer
                  rpm={stats.fans.gpu.speed_rpm}
                  label="GPU"
                  channel="FAN 02"
                  temp={stats.gpu_temp}
                  mode={stats.fans.gpu.mode}
                  theme="purple"
                />
              </div>

              {/* 4 Precision Telemetry Cards */}
              <div className="grid grid-cols-2 xl:grid-cols-4 gap-3 lg:gap-4">
                <TacticalStatCard
                  title="CPU UTILIZATION"
                  subtitle={stats.cpu_name}
                  value={stats.cpu_usage}
                  max={100}
                  unit="%"
                  color="#00e5ff"
                  extra={`${stats.cpu_temp.toFixed(0)}°C PKG`}
                />
                <TacticalStatCard
                  title="GPU CORE CLOCK"
                  subtitle={stats.gpu_name}
                  value={stats.gpu_clock_mhz > 0 ? stats.gpu_clock_mhz : stats.gpu_usage}
                  max={stats.gpu_clock_mhz > 0 ? 2500 : 100}
                  unit={stats.gpu_clock_mhz > 0 ? " MHz" : "%"}
                  color="#b026ff"
                  extra={stats.vram_clock_mhz > 0 ? `${stats.vram_clock_mhz} MHz VRAM` : "Optimus Active"}
                />
                <TacticalStatCard
                  title="PHYSICAL MEMORY"
                  subtitle="DDR5 High-Speed Bus"
                  value={stats.ram_used_gb}
                  max={stats.ram_total_gb || 16}
                  unit=" GB"
                  color="#00f59b"
                  extra={`${((stats.ram_used_gb / stats.ram_total_gb) * 100).toFixed(0)}% ALLOCATED`}
                />
                <TacticalStatCard
                  title="NETWORK STREAM"
                  subtitle="Ethernet / Wi-Fi I/O"
                  value={stats.network.download_bytes_per_sec / 1024}
                  max={1024 * 50}
                  unit=" KB/s"
                  color="#ffaa00"
                  extra={`UP: ${formatSpeed(stats.network.upload_bytes_per_sec)}`}
                />
              </div>
            </div>
          )}

          {/* ══════════════════════════════════════════════════════════════════
              TAB: AEROBLADE 3D (FAN CONTROL)
             ══════════════════════════════════════════════════════════════════ */}
          {activeTab === "fans" && stats && (
            <div className="space-y-6">
              {/* Acer CoolBoost Breaker */}
              <div className="p-4 rounded-xl bg-gradient-to-r from-[#0c121d] to-[#080d16] border border-[#172338] flex items-center justify-between shadow-xl">
                <div className="flex items-center gap-3.5">
                  <div className="w-9 h-9 rounded-lg bg-cyan-950/80 border border-cyan-500/40 flex items-center justify-center">
                    <Zap className="w-5 h-5 text-cyan-400" />
                  </div>
                  <div>
                    <p className="text-sm font-bold font-mono text-slate-100 flex items-center gap-2">
                      ACER COOLBOOST™ OVERDRIVE
                      <span className="text-[9px] px-1.5 py-0.5 rounded bg-[#101828] text-cyan-400 border border-cyan-800/40">
                        CIRCUIT 0x00010007
                      </span>
                    </p>
                    <p className="text-xs text-slate-400 font-mono">
                      Raises baseline fan duty curves by ~10% across all thermal triggers for high ambient environments.
                    </p>
                  </div>
                </div>

                <button
                  onClick={toggleCoolboost}
                  className={`w-12 h-6 rounded-full transition-all duration-300 relative border ${
                    stats.fans.coolboost
                      ? "bg-cyan-500 border-cyan-400 shadow-[0_0_12px_rgba(0,229,255,0.5)]"
                      : "bg-[#111827] border-[#1f293d]"
                  }`}
                >
                  <span
                    className={`absolute top-0.5 w-4.5 h-4.5 rounded-full bg-white transition-all duration-300 ${
                      stats.fans.coolboost ? "left-6" : "left-0.5"
                    }`}
                  />
                </button>
              </div>

              {/* CPU Fan Precision Deck */}
              <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] space-y-4 shadow-xl">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2.5">
                    <Fan className="w-5 h-5 text-cyan-400" />
                    <div>
                      <h3 className="font-bold text-sm font-mono text-slate-100">
                        PRIMARY COOLER // CPU FAN
                      </h3>
                      <p className="text-xs font-mono text-cyan-400">
                        {stats.fans.cpu.speed_rpm > 0 ? `${stats.fans.cpu.speed_rpm.toLocaleString()} RPM (LIVE ACPI)` : "0 RPM (OFF)"}
                      </p>
                    </div>
                  </div>

                  <div className="flex bg-[#0a0e17] p-1 rounded-lg border border-[#172338] gap-1">
                    {(["auto", "custom", "max"] as FanMode[]).map((m) => (
                      <button
                        key={m}
                        onClick={() => setFanMode(1, m, m === "custom" ? cpuSpeed : undefined)}
                        className={`px-3 py-1 text-xs font-mono font-bold rounded transition-all uppercase ${
                          stats.fans.cpu.mode === m
                            ? "bg-cyan-500 text-black shadow-md shadow-cyan-500/30"
                            : "text-slate-400 hover:text-slate-200 hover:bg-[#111726]"
                        }`}
                      >
                        {m}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Potentiometer Slider */}
                <div className="space-y-2 pt-2 border-t border-[#131c2c]">
                  <div className="flex justify-between text-xs font-mono">
                    <span className="text-slate-400 flex items-center gap-1.5">
                      <Sliders className="w-3.5 h-3.5 text-cyan-400" /> MANUAL DUTY CALIBRATION
                    </span>
                    <span className="text-cyan-400 font-bold text-sm">{cpuSpeed}% DUTY</span>
                  </div>

                  <div className="flex items-center gap-3">
                    <button
                      onClick={() => {
                        const val = Math.max(30, cpuSpeed - 5);
                        setCpuSpeed(val);
                        setFanMode(1, "custom", val);
                      }}
                      className="p-1 rounded bg-[#101726] border border-[#1b2638] text-slate-400 hover:text-white"
                    >
                      <Minus className="w-3.5 h-3.5" />
                    </button>

                    <input
                      type="range"
                      min={30}
                      max={100}
                      value={cpuSpeed}
                      onChange={(e) => {
                        const val = +e.target.value;
                        setCpuSpeed(val);
                        setFanMode(1, "custom", val);
                      }}
                      className="flex-1 tactical-slider"
                    />

                    <button
                      onClick={() => {
                        const val = Math.min(100, cpuSpeed + 5);
                        setCpuSpeed(val);
                        setFanMode(1, "custom", val);
                      }}
                      className="p-1 rounded bg-[#101726] border border-[#1b2638] text-slate-400 hover:text-white"
                    >
                      <Plus className="w-3.5 h-3.5" />
                    </button>
                  </div>

                  {/* Calibration tick labels */}
                  <div className="flex justify-between text-[9px] font-mono text-slate-600 px-1 pt-1">
                    <span>30% (QUIET)</span>
                    <span>50%</span>
                    <span>70%</span>
                    <span>85%</span>
                    <span>100% (MAX)</span>
                  </div>
                </div>
              </div>

              {/* GPU Fan Precision Deck */}
              <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] space-y-4 shadow-xl">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2.5">
                    <Fan className="w-5 h-5 text-purple-400" />
                    <div>
                      <h3 className="font-bold text-sm font-mono text-slate-100">
                        SECONDARY COOLER // GPU FAN (INDEX 04)
                      </h3>
                      <p className="text-xs font-mono text-purple-400">
                        {stats.fans.gpu.speed_rpm > 0 ? `${stats.fans.gpu.speed_rpm.toLocaleString()} RPM (LIVE ACPI)` : "0 RPM (OFF)"}
                      </p>
                    </div>
                  </div>

                  <div className="flex bg-[#0a0e17] p-1 rounded-lg border border-[#172338] gap-1">
                    {(["auto", "custom", "max"] as FanMode[]).map((m) => (
                      <button
                        key={m}
                        onClick={() => setFanMode(2, m, m === "custom" ? gpuSpeed : undefined)}
                        className={`px-3 py-1 text-xs font-mono font-bold rounded transition-all uppercase ${
                          stats.fans.gpu.mode === m
                            ? "bg-purple-500 text-black shadow-md shadow-purple-500/30"
                            : "text-slate-400 hover:text-slate-200 hover:bg-[#111726]"
                        }`}
                      >
                        {m}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Potentiometer Slider */}
                <div className="space-y-2 pt-2 border-t border-[#131c2c]">
                  <div className="flex justify-between text-xs font-mono">
                    <span className="text-slate-400 flex items-center gap-1.5">
                      <Sliders className="w-3.5 h-3.5 text-purple-400" /> MANUAL DUTY CALIBRATION
                    </span>
                    <span className="text-purple-400 font-bold text-sm">{gpuSpeed}% DUTY</span>
                  </div>

                  <div className="flex items-center gap-3">
                    <button
                      onClick={() => {
                        const val = Math.max(30, gpuSpeed - 5);
                        setGpuSpeed(val);
                        setFanMode(2, "custom", val);
                      }}
                      className="p-1 rounded bg-[#101726] border border-[#1b2638] text-slate-400 hover:text-white"
                    >
                      <Minus className="w-3.5 h-3.5" />
                    </button>

                    <input
                      type="range"
                      min={30}
                      max={100}
                      value={gpuSpeed}
                      onChange={(e) => {
                        const val = +e.target.value;
                        setGpuSpeed(val);
                        setFanMode(2, "custom", val);
                      }}
                      className="flex-1 tactical-slider tactical-slider-gpu"
                    />

                    <button
                      onClick={() => {
                        const val = Math.min(100, gpuSpeed + 5);
                        setGpuSpeed(val);
                        setFanMode(2, "custom", val);
                      }}
                      className="p-1 rounded bg-[#101726] border border-[#1b2638] text-slate-400 hover:text-white"
                    >
                      <Plus className="w-3.5 h-3.5" />
                    </button>
                  </div>

                  {/* Calibration tick labels */}
                  <div className="flex justify-between text-[9px] font-mono text-slate-600 px-1 pt-1">
                    <span>30% (QUIET)</span>
                    <span>50%</span>
                    <span>70%</span>
                    <span>85%</span>
                    <span>100% (MAX)</span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* ══════════════════════════════════════════════════════════════════
              TAB: PULSAR RGB (LIGHTING)
             ══════════════════════════════════════════════════════════════════ */}
          {activeTab === "lighting" && (
            <div className="space-y-6">
              {/* Presets Bar */}
              <div className="p-4 rounded-xl bg-gradient-to-r from-[#0c121d] to-[#080d16] border border-[#172338] flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-[10px] font-mono font-bold tracking-widest uppercase text-slate-400">
                    PRESET PROFILES:
                  </span>
                  <div className="flex gap-2">
                    {PRESETS.map((p) => (
                      <button
                        key={p.name}
                        onClick={() => applyPresetProfile(p)}
                        className="px-3 py-1.5 rounded bg-[#101726] hover:bg-cyan-950/60 border border-[#1b273d] hover:border-cyan-500/50 text-xs font-mono font-semibold transition-all flex items-center gap-2"
                      >
                        <div className="flex gap-0.5">
                          {p.zones.map((c, idx) => (
                            <span key={idx} className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: c }} />
                          ))}
                        </div>
                        <span>{p.name}</span>
                      </button>
                    ))}
                  </div>
                </div>

                <button
                  onClick={() => applyRgbToAll(zones[activeZone])}
                  className="px-3 py-1.5 rounded bg-cyan-950 text-cyan-300 border border-cyan-500/40 text-xs font-mono font-bold hover:bg-cyan-900 transition-all"
                >
                  APPLY ACTIVE COLOR TO ALL 4 ZONES
                </button>
              </div>

              {/* Realistic Chiclet Keyboard Visualizer */}
              <div className="p-6 rounded-2xl bg-gradient-to-b from-[#0b101a] to-[#070b12] border border-[#172338] space-y-4 shadow-2xl">
                <div className="flex justify-between items-center border-b border-[#131b2a] pb-3">
                  <div className="flex items-center gap-2">
                    <Palette className="w-4 h-4 text-cyan-400" />
                    <h3 className="font-bold text-sm font-mono text-slate-200">
                      PULSAR 4-ZONE CHICLET KEYBOARD DECK
                    </h3>
                  </div>
                  <span className="text-xs font-mono text-cyan-400">
                    Zone {activeZone + 1} Selected // {zones[activeZone].toUpperCase()}
                  </span>
                </div>

                {/* Keyboard Grid Layout */}
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-3.5 p-3 rounded-xl bg-[#05080e] border border-[#101726]">
                  {/* Zone 1: WASD & Left Modifiers */}
                  <div
                    onClick={() => setActiveZone(0)}
                    style={{
                      borderColor: activeZone === 0 ? "#00e5ff" : "rgba(255,255,255,0.1)",
                      boxShadow: activeZone === 0 ? `0 0 20px ${zones[0]}44` : undefined,
                    }}
                    className="p-3.5 rounded-xl border-2 cursor-pointer transition-all flex flex-col justify-between h-40 relative group"
                  >
                    <div className="flex justify-between items-start">
                      <span className="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-black/60 text-slate-300">
                        ZONE 01 // WASD
                      </span>
                      <span className="w-3.5 h-3.5 rounded-full border border-white/50" style={{ backgroundColor: zones[0] }} />
                    </div>

                    {/* Miniature Chiclets */}
                    <div className="space-y-1 my-auto pointer-events-none opacity-80 group-hover:opacity-100">
                      <div className="flex justify-center gap-1">
                        <div className="w-7 h-6 rounded bg-[#131a29] border border-cyan-400/40 text-[9px] font-mono font-bold flex items-center justify-center text-cyan-300">W</div>
                      </div>
                      <div className="flex justify-center gap-1">
                        <div className="w-7 h-6 rounded bg-[#131a29] border border-cyan-400/40 text-[9px] font-mono font-bold flex items-center justify-center text-cyan-300">A</div>
                        <div className="w-7 h-6 rounded bg-[#131a29] border border-cyan-400/40 text-[9px] font-mono font-bold flex items-center justify-center text-cyan-300">S</div>
                        <div className="w-7 h-6 rounded bg-[#131a29] border border-cyan-400/40 text-[9px] font-mono font-bold flex items-center justify-center text-cyan-300">D</div>
                      </div>
                    </div>

                    <div className="h-1 rounded-full" style={{ backgroundColor: zones[0], boxShadow: `0 0 10px ${zones[0]}` }} />
                  </div>

                  {/* Zone 2: Center Alpha */}
                  <div
                    onClick={() => setActiveZone(1)}
                    style={{
                      borderColor: activeZone === 1 ? "#00e5ff" : "rgba(255,255,255,0.1)",
                      boxShadow: activeZone === 1 ? `0 0 20px ${zones[1]}44` : undefined,
                    }}
                    className="p-3.5 rounded-xl border-2 cursor-pointer transition-all flex flex-col justify-between h-40 relative group"
                  >
                    <div className="flex justify-between items-start">
                      <span className="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-black/60 text-slate-300">
                        ZONE 02 // ALPHA-C
                      </span>
                      <span className="w-3.5 h-3.5 rounded-full border border-white/50" style={{ backgroundColor: zones[1] }} />
                    </div>

                    <div className="space-y-1 my-auto pointer-events-none opacity-80 group-hover:opacity-100">
                      <div className="flex justify-center gap-1">
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">T</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">Y</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">U</div>
                      </div>
                      <div className="flex justify-center gap-1">
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">G</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">H</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">J</div>
                      </div>
                    </div>

                    <div className="h-1 rounded-full" style={{ backgroundColor: zones[1], boxShadow: `0 0 10px ${zones[1]}` }} />
                  </div>

                  {/* Zone 3: Right Alpha + Navigation */}
                  <div
                    onClick={() => setActiveZone(2)}
                    style={{
                      borderColor: activeZone === 2 ? "#00e5ff" : "rgba(255,255,255,0.1)",
                      boxShadow: activeZone === 2 ? `0 0 20px ${zones[2]}44` : undefined,
                    }}
                    className="p-3.5 rounded-xl border-2 cursor-pointer transition-all flex flex-col justify-between h-40 relative group"
                  >
                    <div className="flex justify-between items-start">
                      <span className="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-black/60 text-slate-300">
                        ZONE 03 // NAV+ENTER
                      </span>
                      <span className="w-3.5 h-3.5 rounded-full border border-white/50" style={{ backgroundColor: zones[2] }} />
                    </div>

                    <div className="space-y-1 my-auto pointer-events-none opacity-80 group-hover:opacity-100">
                      <div className="flex justify-center gap-1">
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">O</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">P</div>
                        <div className="w-8 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">ENTER</div>
                      </div>
                      <div className="flex justify-center gap-1">
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">K</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">L</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">▲</div>
                      </div>
                    </div>

                    <div className="h-1 rounded-full" style={{ backgroundColor: zones[2], boxShadow: `0 0 10px ${zones[2]}` }} />
                  </div>

                  {/* Zone 4: Numpad & Predator Key */}
                  <div
                    onClick={() => setActiveZone(3)}
                    style={{
                      borderColor: activeZone === 3 ? "#00e5ff" : "rgba(255,255,255,0.1)",
                      boxShadow: activeZone === 3 ? `0 0 20px ${zones[3]}44` : undefined,
                    }}
                    className="p-3.5 rounded-xl border-2 cursor-pointer transition-all flex flex-col justify-between h-40 relative group"
                  >
                    <div className="flex justify-between items-start">
                      <span className="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-black/60 text-slate-300">
                        ZONE 04 // 10-KEY
                      </span>
                      <span className="w-3.5 h-3.5 rounded-full border border-white/50" style={{ backgroundColor: zones[3] }} />
                    </div>

                    <div className="space-y-1 my-auto pointer-events-none opacity-80 group-hover:opacity-100">
                      <div className="flex justify-center gap-1">
                        <div className="w-6 h-5 rounded bg-cyan-950 border border-cyan-400/40 text-[8px] font-mono font-bold flex items-center justify-center text-cyan-300">PS</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">7</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">8</div>
                      </div>
                      <div className="flex justify-center gap-1">
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">4</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">5</div>
                        <div className="w-6 h-5 rounded bg-[#131a29] text-[8px] font-mono flex items-center justify-center text-slate-300">6</div>
                      </div>
                    </div>

                    <div className="h-1 rounded-full" style={{ backgroundColor: zones[3], boxShadow: `0 0 10px ${zones[3]}` }} />
                  </div>
                </div>

                {/* Color Customizer Palette for Active Zone */}
                <div className="p-4 rounded-xl bg-[#090d16] border border-[#172338] flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-xs font-mono font-semibold text-slate-300">
                      CALIBRATE ZONE 0{activeZone + 1}:
                    </span>
                    <input
                      type="color"
                      value={zones[activeZone]}
                      onChange={(e) => applyRgb(activeZone, e.target.value)}
                      className="w-10 h-8 rounded cursor-pointer bg-transparent border-0"
                    />
                    <span className="text-xs font-mono font-bold text-cyan-400">
                      {zones[activeZone].toUpperCase()}
                    </span>
                  </div>

                  <div className="flex items-center gap-2">
                    {["#00AEC7", "#00E5FF", "#39FF14", "#E0007B", "#7B00FF", "#FF3300", "#FFAA00", "#FFFFFF"].map((hex) => (
                      <button
                        key={hex}
                        onClick={() => applyRgb(activeZone, hex)}
                        style={{ backgroundColor: hex }}
                        className="w-6 h-6 rounded-md border border-white/20 hover:scale-110 hover:border-white transition-all shadow-sm"
                      />
                    ))}
                  </div>
                </div>
              </div>

              {/* Dynamic Effects Deck */}
              <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] space-y-4 shadow-xl">
                <div className="flex justify-between items-center border-b border-[#131b2a] pb-3">
                  <h3 className="font-bold text-sm font-mono text-slate-200">
                    DYNAMIC LIGHTING CIRCUIT (ACPI FIRMWARE ENGINE)
                  </h3>
                  <span className="text-xs font-mono text-slate-400">Fixed 16-Byte Hardware Array</span>
                </div>

                <div className="grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-7 gap-2">
                  {[
                    { id: "static", label: "Static" },
                    { id: "breathing", label: "Breathing" },
                    { id: "wave", label: "Wave" },
                    { id: "zoom", label: "Zoom" },
                    { id: "shift", label: "Shift" },
                    { id: "neon", label: "Neon Cycle" },
                    { id: "rainbow", label: "Rainbow" },
                  ].map(({ id, label }) => (
                    <button
                      key={id}
                      onClick={() => applyDynamicEffect(id)}
                      className={`p-3 rounded-lg text-xs font-mono font-bold transition-all border text-center ${
                        selectedEffect === id
                          ? "bg-cyan-500 text-black border-cyan-400 shadow-[0_0_12px_rgba(0,229,255,0.4)]"
                          : "bg-[#0b101b] text-slate-400 border-[#192439] hover:text-slate-200 hover:bg-[#111726]"
                      }`}
                    >
                      {label}
                    </button>
                  ))}
                </div>

                {/* Speed & Direction Grids */}
                <div className="grid grid-cols-2 gap-4 pt-2">
                  <div className="p-3 rounded-lg bg-[#090d16] border border-[#141d2d] space-y-2">
                    <span className="text-[11px] font-mono text-slate-400 font-semibold">VELOCITY RATE:</span>
                    <div className="grid grid-cols-3 gap-2">
                      {[1, 2, 3].map((s) => (
                        <button
                          key={s}
                          onClick={() => { setEffectSpeed(s); applyDynamicEffect(selectedEffect, s, effectDirection); }}
                          className={`py-1.5 text-xs font-mono font-bold rounded border ${
                            effectSpeed === s ? "bg-cyan-950 text-cyan-300 border-cyan-400" : "bg-[#0f1522] text-slate-400 border-[#1b2638]"
                          }`}
                        >
                          {s === 1 ? "1X SLOW" : s === 2 ? "2X NORMAL" : "3X FAST"}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="p-3 rounded-lg bg-[#090d16] border border-[#141d2d] space-y-2">
                    <span className="text-[11px] font-mono text-slate-400 font-semibold">PROPAGATION VECTOR:</span>
                    <div className="grid grid-cols-4 gap-2">
                      {[
                        { id: 1, label: "EAST →" },
                        { id: 2, label: "WEST ←" },
                        { id: 3, label: "SOUTH ↓" },
                        { id: 4, label: "NORTH ↑" },
                      ].map(({ id, label }) => (
                        <button
                          key={id}
                          onClick={() => { setEffectDirection(id); applyDynamicEffect(selectedEffect, effectSpeed, id); }}
                          className={`py-1.5 text-xs font-mono font-bold rounded border ${
                            effectDirection === id ? "bg-cyan-950 text-cyan-300 border-cyan-400" : "bg-[#0f1522] text-slate-400 border-[#1b2638]"
                          }`}
                        >
                          {label}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>

                {/* Backlight Brightness Slider */}
                <div className="p-4 rounded-lg bg-[#090d16] border border-[#141d2d] space-y-2.5">
                  <div className="flex justify-between text-xs font-mono">
                    <span className="text-slate-300 font-semibold">KEYBOARD BACKLIGHT INTENSITY</span>
                    <span className="text-cyan-400 font-bold">{kbBrightness}%</span>
                  </div>
                  <input
                    type="range"
                    min={0}
                    max={100}
                    value={kbBrightness}
                    onChange={(e) => handleBrightnessChange(+e.target.value)}
                    className="w-full tactical-slider"
                  />
                  <div className="flex justify-between text-[9px] font-mono text-slate-600">
                    <span>0% (OFF)</span>
                    <span>25%</span>
                    <span>50%</span>
                    <span>75%</span>
                    <span>100% (FULL)</span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* ══════════════════════════════════════════════════════════════════
              TAB: DIAGNOSTICS & TELEMETRY
             ══════════════════════════════════════════════════════════════════ */}
          {activeTab === "monitoring" && stats && (
            <div className="space-y-6">
              {/* Primary Hardware Bus Cards */}
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                {/* CPU Telemetry Card */}
                <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] space-y-4">
                  <div className="flex justify-between items-start">
                    <div className="flex items-center gap-2.5">
                      <Cpu className="w-5 h-5 text-cyan-400" />
                      <div>
                        <h4 className="font-bold text-sm font-mono text-slate-100">CPU COMPUTATION CORE</h4>
                        <p className="text-xs text-slate-500 font-mono truncate max-w-[220px]">{stats.cpu_name}</p>
                      </div>
                    </div>
                    <span className="text-2xl font-black font-mono text-cyan-400">
                      {stats.cpu_usage.toFixed(1)}%
                    </span>
                  </div>

                  <div className="grid grid-cols-2 gap-3 pt-3 border-t border-[#131c2c] text-xs font-mono">
                    <div className="p-2.5 rounded bg-[#090d16] border border-[#141d2d]">
                      <span className="text-[10px] text-slate-500 block">PACKAGE TEMPERATURE</span>
                      <strong className="text-sm text-orange-400">{stats.cpu_temp.toFixed(0)}°C</strong>
                    </div>
                    <div className="p-2.5 rounded bg-[#090d16] border border-[#141d2d]">
                      <span className="text-[10px] text-slate-500 block">FAN 01 SPEED</span>
                      <strong className="text-sm text-cyan-400">{stats.fans.cpu.speed_rpm} RPM</strong>
                    </div>
                  </div>
                </div>

                {/* GPU Telemetry Card */}
                <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] space-y-4">
                  <div className="flex justify-between items-start">
                    <div className="flex items-center gap-2.5">
                      <Flame className="w-5 h-5 text-purple-400" />
                      <div>
                        <h4 className="font-bold text-sm font-mono text-slate-100">GPU GRAPHICS ENGINE</h4>
                        <p className="text-xs text-slate-500 font-mono truncate max-w-[220px]">{stats.gpu_name}</p>
                      </div>
                    </div>
                    <span className="text-2xl font-black font-mono text-purple-400">
                      {stats.gpu_clock_mhz > 0 ? `${stats.gpu_clock_mhz} MHz` : "OPTIMUS"}
                    </span>
                  </div>

                  <div className="grid grid-cols-2 gap-3 pt-3 border-t border-[#131c2c] text-xs font-mono">
                    <div className="p-2.5 rounded bg-[#090d16] border border-[#141d2d]">
                      <span className="text-[10px] text-slate-500 block">CORE TEMPERATURE</span>
                      <strong className="text-sm text-purple-400">{stats.gpu_temp.toFixed(0)}°C</strong>
                    </div>
                    <div className="p-2.5 rounded bg-[#090d16] border border-[#141d2d]">
                      <span className="text-[10px] text-slate-500 block">VRAM FREQUENCY</span>
                      <strong className="text-sm text-teal-400">{stats.vram_clock_mhz > 0 ? `${stats.vram_clock_mhz} MHz` : "STANDBY"}</strong>
                    </div>
                  </div>
                </div>
              </div>

              {/* Real-Time Network Oscillograph */}
              <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] space-y-4 shadow-xl">
                <div className="flex justify-between items-center border-b border-[#131c2c] pb-3">
                  <div className="flex items-center gap-2">
                    <Activity className="w-4 h-4 text-emerald-400" />
                    <h3 className="font-bold text-sm font-mono text-slate-200">
                      MULTI-CHANNEL NETWORK OSCILLOSCOPE (ETHERNET & WI-FI)
                    </h3>
                  </div>
                  <div className="flex items-center gap-4 text-xs font-mono">
                    <span className="text-emerald-400 flex items-center gap-1 font-bold">
                      <ArrowDown className="w-3.5 h-3.5" /> RX: {formatSpeed(stats.network.download_bytes_per_sec)}
                    </span>
                    <span className="text-cyan-400 flex items-center gap-1 font-bold">
                      <ArrowUp className="w-3.5 h-3.5" /> TX: {formatSpeed(stats.network.upload_bytes_per_sec)}
                    </span>
                  </div>
                </div>

                {/* Oscillograph bars */}
                <div className="h-20 flex items-end gap-1.5 p-3 bg-[#06090f] rounded-lg border border-[#121927]">
                  {netHistory.down.map((val, idx) => {
                    const maxVal = Math.max(...netHistory.down, 1024 * 60);
                    const barHeight = Math.max(4, (val / maxVal) * 56);
                    return (
                      <div key={idx} className="flex-1 flex flex-col justify-end items-center h-full">
                        <div
                          className="w-full bg-gradient-to-t from-emerald-600/40 to-emerald-400 rounded-t transition-all duration-300"
                          style={{ height: `${barHeight}px`, boxShadow: "0 0 6px rgba(52,211,153,0.3)" }}
                        />
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* Physical Memory Allocation */}
              <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] space-y-3">
                <div className="flex justify-between text-xs font-mono text-slate-300">
                  <span>PHYSICAL MEMORY POOL</span>
                  <span className="text-emerald-400 font-bold">
                    {stats.ram_used_gb.toFixed(1)} GB / {stats.ram_total_gb.toFixed(1)} GB ALLOCATED
                  </span>
                </div>
                <div className="h-2 rounded-full bg-[#0a0f19] border border-[#162134] overflow-hidden">
                  <div
                    className="h-full rounded-full bg-gradient-to-r from-emerald-500 to-teal-400 transition-all duration-700"
                    style={{ width: `${Math.min(100, (stats.ram_used_gb / stats.ram_total_gb) * 100)}%` }}
                  />
                </div>
              </div>
            </div>
          )}

          {/* ══════════════════════════════════════════════════════════════════
              TAB: GAME SYNC
             ══════════════════════════════════════════════════════════════════ */}
          {activeTab === "gamesync" && (
            <div className="space-y-6">
              <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] flex items-center justify-between shadow-xl">
                <div>
                  <h3 className="font-bold text-sm font-mono text-slate-100 flex items-center gap-2">
                    <Gamepad2 className="w-5 h-5 text-cyan-400" /> AUTONOMIC GAME SYNC ENGINE
                  </h3>
                  <p className="text-xs text-slate-400 font-mono mt-1">
                    Detects active game processes and automatically applies configured thermal & illumination profiles.
                  </p>
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={rescanGames}
                    disabled={scanningGames}
                    className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-mono font-bold bg-[#101726] text-slate-200 hover:bg-[#152033] border border-[#1b273d] transition-all"
                  >
                    <RefreshCw className={`w-3.5 h-3.5 ${scanningGames ? "animate-spin" : ""}`} />
                    <span>{scanningGames ? "SCANNING LIBRARIES…" : "RESCAN LIBRARIES"}</span>
                  </button>
                  <button
                    onClick={() => setShowAddGame(true)}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-black bg-cyan-500 text-black hover:bg-cyan-400 transition-all uppercase"
                  >
                    <span>+ ADD GAME</span>
                  </button>
                </div>
              </div>

              {/* Game list */}
              <div className="space-y-3">
                {games.length === 0 ? (
                  <div className="p-8 text-center text-slate-500 font-mono text-sm border border-dashed border-[#182338] rounded-xl">
                    No games detected yet. Click "RESCAN LIBRARIES" or add custom game executable.
                  </div>
                ) : (
                  games.map((game) => (
                    <div
                      key={game.id}
                      className="p-4 rounded-xl bg-gradient-to-r from-[#0c121d] to-[#080d16] border border-[#172338] flex items-center justify-between hover:border-cyan-500/40 transition-all shadow-md"
                    >
                      <div className="flex items-center gap-3.5">
                        <div className="w-10 h-10 rounded-lg bg-[#0e1626] border border-cyan-500/30 flex items-center justify-center text-cyan-400">
                          <Gamepad2 className="w-5 h-5" />
                        </div>
                        <div>
                          <p className="font-bold text-sm text-slate-100 font-mono">{game.name}</p>
                          <p className="text-[11px] font-mono text-slate-500">{game.exe_name}</p>
                        </div>
                      </div>

                      <div className="flex items-center gap-3">
                        <span className="px-2.5 py-1 rounded bg-red-950/60 text-red-400 border border-red-800/40 font-mono text-xs font-bold uppercase">
                          {game.performance_mode}
                        </span>
                        <span className="px-2.5 py-1 rounded bg-cyan-950/60 text-cyan-400 border border-cyan-800/40 font-mono text-xs font-semibold uppercase">
                          FANS {game.fan_mode}
                        </span>
                        <button
                          onClick={() => {
                            const updated = games.map((g) => g.id === game.id ? { ...g, enabled: !g.enabled } : g);
                            setGames(updated);
                          }}
                          className={`w-10 h-5 rounded-full relative transition-all border ${
                            game.enabled ? "bg-cyan-500 border-cyan-400" : "bg-slate-800 border-slate-700"
                          }`}
                        >
                          <span className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all ${
                            game.enabled ? "left-5" : "left-0.5"
                          }`} />
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}

          {/* ══════════════════════════════════════════════════════════════════
              TAB: LAUNCHPAD (APP CENTER)
             ══════════════════════════════════════════════════════════════════ */}
          {activeTab === "appcenter" && (
            <div className="space-y-6">
              <div className="p-5 rounded-xl bg-gradient-to-b from-[#0c121d] to-[#080d16] border border-[#172338] shadow-xl">
                <h3 className="font-bold text-sm font-mono text-slate-100 flex items-center gap-2">
                  <Grid className="w-5 h-5 text-cyan-400" /> TACTICAL UTILITY LAUNCHPAD
                </h3>
                <p className="text-xs text-slate-400 font-mono mt-1">
                  1-Click executable launcher for OEM hardware tools, GPU utilities, and streaming software.
                </p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {apps.map((app) => (
                  <div
                    key={app.id}
                    className={`p-5 rounded-xl border transition-all flex flex-col justify-between h-40 ${
                      app.installed
                        ? "bg-[#0b1019] border-[#182438] hover:border-cyan-500/50 shadow-lg"
                        : "bg-[#070a10] border-[#131b28] opacity-50"
                    }`}
                  >
                    <div>
                      <div className="flex justify-between items-start mb-2">
                        <span className="text-[9px] uppercase font-mono px-2 py-0.5 rounded bg-[#101726] border border-[#1b2638] text-slate-400 font-bold">
                          {app.category}
                        </span>
                        <span className={`text-[9px] font-mono px-2 py-0.5 rounded font-bold ${
                          app.installed ? "bg-emerald-950 text-emerald-400 border border-emerald-800/40" : "bg-slate-900 text-slate-500"
                        }`}>
                          {app.installed ? "READY" : "NOT DETECTED"}
                        </span>
                      </div>
                      <h4 className="font-bold text-slate-100 text-sm font-mono">{app.name}</h4>
                      <p className="text-xs text-slate-400 mt-1 line-clamp-2">{app.description}</p>
                    </div>

                    <div className="pt-2">
                      <button
                        onClick={() => launchAppItem(app.exe_path)}
                        disabled={!app.installed}
                        className={`w-full py-1.5 px-3 rounded text-xs font-mono font-bold flex items-center justify-center gap-1.5 transition-all uppercase ${
                          app.installed
                            ? "bg-cyan-500 text-black hover:bg-cyan-400 shadow-md shadow-cyan-500/20"
                            : "bg-[#111724] text-slate-600 cursor-not-allowed border border-[#162134]"
                        }`}
                      >
                        <Play className="w-3.5 h-3.5 fill-current" /> LAUNCH UTILITY
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </main>

      {/* ── Settings Modal ── */}
      {showSettings && (
        <div className="fixed inset-0 bg-black/85 backdrop-blur-md z-50 flex items-center justify-center p-6">
          <div className="w-full max-w-lg bg-[#0a0f19] border border-cyan-500/40 rounded-2xl p-6 space-y-6 shadow-[0_0_50px_rgba(0,0,0,0.9)]">
            <div className="flex justify-between items-center border-b border-[#152033] pb-4">
              <h2 className="text-base font-bold font-mono text-slate-100 flex items-center gap-2">
                <SettingsIcon className="w-5 h-5 text-cyan-400" /> SYSTEM ARCHITECTURE & ACPI FIRMWARE
              </h2>
              <button
                onClick={() => setShowSettings(false)}
                className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-3.5">
              {/* Windows Key Lock */}
              <div className="flex items-center justify-between p-3.5 rounded-xl bg-[#06090f] border border-[#141d2d]">
                <div>
                  <p className="text-xs font-bold font-mono text-slate-200">WINDOWS KEY GAMING LOCK</p>
                  <p className="text-[11px] text-slate-400 font-mono">Disables Windows Key via ACPI Profile & Low-Level WH_KEYBOARD Hook.</p>
                </div>
                <button
                  onClick={toggleWinKeyLock}
                  className={`w-11 h-6 rounded-full relative transition-all border ${
                    stats?.settings.win_key_locked ? "bg-cyan-500 border-cyan-400" : "bg-slate-800 border-slate-700"
                  }`}
                >
                  <span className={`absolute top-0.5 w-5 h-5 rounded-full bg-white transition-all ${
                    stats?.settings.win_key_locked ? "left-5" : "left-0.5"
                  }`} />
                </button>
              </div>

              {/* Sticky Keys Lock */}
              <div className="flex items-center justify-between p-3.5 rounded-xl bg-[#06090f] border border-[#141d2d]">
                <div>
                  <p className="text-xs font-bold font-mono text-slate-200">STICKY KEYS POPUP SUPPRESSION</p>
                  <p className="text-[11px] text-slate-400 font-mono">Locks 5x Shift key accessibility prompt via Win32 SystemParametersInfo.</p>
                </div>
                <button
                  onClick={toggleStickyKeys}
                  className={`w-11 h-6 rounded-full relative transition-all border ${
                    !stats?.settings.sticky_keys_enabled ? "bg-cyan-500 border-cyan-400" : "bg-slate-800 border-slate-700"
                  }`}
                >
                  <span className={`absolute top-0.5 w-5 h-5 rounded-full bg-white transition-all ${
                    !stats?.settings.sticky_keys_enabled ? "left-5" : "left-0.5"
                  }`} />
                </button>
              </div>

              {/* LCD OverDrive */}
              <div className="flex items-center justify-between p-3.5 rounded-xl bg-[#06090f] border border-[#141d2d]">
                <div>
                  <p className="text-xs font-bold font-mono text-slate-200">LCD 3ms PANEL OVERDRIVE</p>
                  <p className="text-[11px] text-slate-400 font-mono">Voltage overdrive on Acer display panel for reduced ghosting.</p>
                </div>
                <button
                  onClick={toggleLcdOverdrive}
                  className={`w-11 h-6 rounded-full relative transition-all border ${
                    stats?.settings.lcd_overdrive ? "bg-cyan-500 border-cyan-400" : "bg-slate-800 border-slate-700"
                  }`}
                >
                  <span className={`absolute top-0.5 w-5 h-5 rounded-full bg-white transition-all ${
                    stats?.settings.lcd_overdrive ? "left-5" : "left-0.5"
                  }`} />
                </button>
              </div>

              {/* Boot Sound */}
              <div className="flex items-center justify-between p-3.5 rounded-xl bg-[#06090f] border border-[#141d2d]">
                <div>
                  <p className="text-xs font-bold font-mono text-slate-200">BIOS BOOT ROAR CHIME</p>
                  <p className="text-[11px] text-slate-400 font-mono">Plays Predator logo chime during BIOS power-on self-test.</p>
                </div>
                <button
                  onClick={toggleBootSound}
                  className={`w-11 h-6 rounded-full relative transition-all border ${
                    stats?.settings.boot_sound ? "bg-cyan-500 border-cyan-400" : "bg-slate-800 border-slate-700"
                  }`}
                >
                  <span className={`absolute top-0.5 w-5 h-5 rounded-full bg-white transition-all ${
                    stats?.settings.boot_sound ? "left-5" : "left-0.5"
                  }`} />
                </button>
              </div>
            </div>

            <div className="p-3 rounded-xl bg-[#060b14] border border-cyan-500/30 text-xs font-mono text-slate-400 flex items-center gap-3">
              <ShieldAlert className="w-5 h-5 text-cyan-400 shrink-0" />
              <span>
                Hardware commands execute directly on root\wmi:AcerGamingFunction with kernel elevation.
              </span>
            </div>
          </div>
        </div>
      )}

      {/* ── Add Game Modal ── */}
      {showAddGame && (
        <div className="fixed inset-0 bg-black/85 backdrop-blur-md z-50 flex items-center justify-center p-6">
          <div className="w-full max-w-md bg-[#0a0f19] border border-cyan-500/40 rounded-2xl p-6 space-y-4 shadow-2xl">
            <div className="flex justify-between items-center border-b border-[#152033] pb-3">
              <h3 className="font-bold text-sm font-mono text-slate-100">REGISTER TACTICAL GAME PROFILE</h3>
              <button onClick={() => setShowAddGame(false)} className="text-slate-400 hover:text-white">
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-3 font-mono">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">GAME DISPLAY TITLE</label>
                <input
                  type="text"
                  placeholder="e.g. Cyberpunk 2077"
                  value={newGameName}
                  onChange={(e) => setNewGameName(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg bg-[#06090f] border border-[#1b2638] text-xs text-white focus:border-cyan-400 outline-none"
                />
              </div>
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">PROCESS EXECUTABLE NAME</label>
                <input
                  type="text"
                  placeholder="e.g. Cyberpunk2077.exe"
                  value={newGameExe}
                  onChange={(e) => setNewGameExe(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg bg-[#06090f] border border-[#1b2638] text-xs text-white focus:border-cyan-400 outline-none"
                />
              </div>
            </div>
            <div className="flex justify-end gap-2 pt-2">
              <button
                onClick={() => setShowAddGame(false)}
                className="px-4 py-2 rounded-lg text-xs font-mono font-bold bg-[#101726] text-slate-300"
              >
                CANCEL
              </button>
              <button
                onClick={addCustomGame}
                className="px-4 py-2 rounded-lg text-xs font-mono font-black bg-cyan-500 text-black uppercase"
              >
                SAVE FLIGHT PROFILE
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ── Official Introduction of Modes Modal (Image #2) ── */}
      {showModesIntro && (
        <div className="fixed inset-0 bg-black/85 backdrop-blur-md z-50 flex items-center justify-center p-6">
          <div className="w-full max-w-xl bg-[#090d16] border-2 border-cyan-400 rounded-2xl p-7 space-y-6 shadow-[0_0_50px_rgba(0,229,255,0.3)] relative">
            {/* Header */}
            <div className="flex justify-between items-center border-b border-[#152033] pb-4">
              <h2 className="text-lg font-bold font-mono tracking-wider text-white">
                Introduction of Modes
              </h2>
              <button
                onClick={() => setShowModesIntro(false)}
                className="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-slate-800 transition-all cursor-pointer"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* 4 Mode Rows from Image #2 */}
            <div className="space-y-4">
              {/* Quiet */}
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-[#0e1524] border border-slate-700 flex items-center justify-center text-slate-300 shrink-0">
                  <QuietIcon className="w-7 h-7" />
                </div>
                <div className="pl-3 border-l-4 border-slate-300 flex-1">
                  <h3 className="text-sm font-bold font-mono text-white">Quiet</h3>
                  <p className="text-xs text-slate-400 font-sans leading-relaxed mt-0.5">
                    Keep the PC as quiet as possible. For light use, such as meetings, E-learning or browsing.
                  </p>
                </div>
              </div>

              {/* Default */}
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-[#0e1524] border border-cyan-500/40 flex items-center justify-center text-cyan-400 shrink-0">
                  <DefaultIcon className="w-7 h-7" />
                </div>
                <div className="pl-3 border-l-4 border-[#00e5ff] flex-1">
                  <h3 className="text-sm font-bold font-mono text-white">Default</h3>
                  <p className="text-xs text-slate-400 font-sans leading-relaxed mt-0.5">
                    Recommended for standard, everyday use, such as word processing, office work, etc.
                  </p>
                </div>
              </div>

              {/* Extreme */}
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-[#0e1524] border border-orange-500/40 flex items-center justify-center text-orange-400 shrink-0">
                  <ExtremeIcon className="w-7 h-7" />
                </div>
                <div className="pl-3 border-l-4 border-[#ff8800] flex-1">
                  <h3 className="text-sm font-bold font-mono text-white">Extreme</h3>
                  <p className="text-xs text-slate-400 font-sans leading-relaxed mt-0.5">
                    For gaming. CPU & GPU overclocked with throttled fan speed.
                  </p>
                </div>
              </div>

              {/* Turbo */}
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-[#0e1524] border border-red-500/40 flex items-center justify-center text-red-500 shrink-0">
                  <TurboIcon className="w-7 h-7" />
                </div>
                <div className="pl-3 border-l-4 border-[#ff1100] flex-1">
                  <h3 className="text-sm font-bold font-mono text-white">Turbo</h3>
                  <p className="text-xs text-slate-400 font-sans leading-relaxed mt-0.5">
                    For unleashed gaming. Maximize overclock performance and fan speed.
                  </p>
                </div>
              </div>
            </div>

            {/* Footer with Miniature Keyboard Icons and OK button */}
            <div className="flex items-center justify-between pt-4 border-t border-[#152033]">
              <div className="flex items-center gap-3">
                <div className="grid grid-cols-2 gap-1 shrink-0">
                  <div className="w-6 h-3 rounded-sm bg-slate-400/40 border border-slate-300" />
                  <div className="w-6 h-3 rounded-sm bg-cyan-500/40 border border-cyan-400" />
                  <div className="w-6 h-3 rounded-sm bg-orange-500/40 border border-orange-400" />
                  <div className="w-6 h-3 rounded-sm bg-red-600/40 border border-red-500" />
                </div>
                <p className="text-[11px] text-slate-400 font-sans leading-tight max-w-xs">
                  The Keyboard and Light Bar will show the color effect for a while when changing mode.
                </p>
              </div>

              <button
                onClick={() => setShowModesIntro(false)}
                className="px-6 py-2 rounded-lg bg-[#0e1524] hover:bg-cyan-500 text-white hover:text-black border border-cyan-500/60 font-mono font-bold text-xs uppercase tracking-wider transition-all cursor-pointer shadow-lg"
              >
                OK
              </button>
            </div>
          </div>
        </div>
      )}

    </div>
  );
}
