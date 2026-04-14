import { ref, watchEffect } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const STORAGE_KEY = 'spm_theme'
const ACCENT_KEY = 'spm_accent_color'

type ThemePreference = 'light' | 'dark' | 'system'

export interface AccentColor {
  primary: string
  hover: string
  pressed: string
}

export interface PresetColor {
  name: string
  primary: string
  hover: string
  pressed: string
}

export const PRESET_COLORS: PresetColor[] = [
  { name: 'Ocean Blue', primary: '#3b82f6', hover: '#60a5fa', pressed: '#2563eb' },
  { name: 'Teal', primary: '#14b8a6', hover: '#2dd4bf', pressed: '#0d9488' },
  { name: 'Emerald', primary: '#10b981', hover: '#34d399', pressed: '#059669' },
  { name: 'Amber', primary: '#f59e0b', hover: '#fbbf24', pressed: '#d97706' },
  { name: 'Rose', primary: '#f43f5e', hover: '#fb7185', pressed: '#e11d48' },
  { name: 'Violet', primary: '#8b5cf6', hover: '#a78bfa', pressed: '#7c3aed' },
  { name: 'Slate', primary: '#64748b', hover: '#94a3b8', pressed: '#475569' },
]

const DEFAULT_ACCENT: AccentColor = PRESET_COLORS[0]

const isDark = ref(false)
const accentColor = ref<AccentColor>({ ...DEFAULT_ACCENT })

function getSystemPreference(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function resolveIsDark(preference: ThemePreference): boolean {
  if (preference === 'dark') return true
  if (preference === 'light') return false
  return getSystemPreference()
}

function syncTauriTheme(dark: boolean) {
  getCurrentWindow().setTheme(dark ? 'dark' : 'light').catch(() => {})
}

function initTheme() {
  const stored = localStorage.getItem(STORAGE_KEY) as ThemePreference | null
  const preference: ThemePreference = stored || 'system'
  isDark.value = resolveIsDark(preference)

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  mediaQuery.addEventListener('change', (e) => {
    const current = localStorage.getItem(STORAGE_KEY) as ThemePreference | null
    if (!current || current === 'system') {
      isDark.value = e.matches
    }
  })
}

function initAccentColor() {
  const stored = localStorage.getItem(ACCENT_KEY)
  if (stored) {
    try {
      const parsed = JSON.parse(stored)
      if (parsed.primary && parsed.hover && parsed.pressed) {
        accentColor.value = parsed
        return
      }
    } catch {
      // fall through to default
    }
  }
  accentColor.value = { ...DEFAULT_ACCENT }
}

function toggleTheme() {
  isDark.value = !isDark.value
  localStorage.setItem(STORAGE_KEY, isDark.value ? 'dark' : 'light')
}

function setAccentColor(color: AccentColor) {
  accentColor.value = { ...color }
  localStorage.setItem(ACCENT_KEY, JSON.stringify(accentColor.value))
}

function isPresetSelected(preset: PresetColor): boolean {
  return accentColor.value.primary === preset.primary
}

function isPreset(color: AccentColor): boolean {
  return PRESET_COLORS.some(p => p.primary === color.primary)
}

/** Lighten a hex color by mixing with white */
function lighten(hex: string, amount: number): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  const mix = (c: number) => Math.round(c + (255 - c) * amount)
  return `#${mix(r).toString(16).padStart(2, '0')}${mix(g).toString(16).padStart(2, '0')}${mix(b).toString(16).padStart(2, '0')}`
}

/** Darken a hex color by mixing with black */
function darken(hex: string, amount: number): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  const mix = (c: number) => Math.round(c * (1 - amount))
  return `#${mix(r).toString(16).padStart(2, '0')}${mix(g).toString(16).padStart(2, '0')}${mix(b).toString(16).padStart(2, '0')}`
}

function setCustomAccentColor(hex: string) {
  const color: AccentColor = {
    primary: hex,
    hover: lighten(hex, 0.2),
    pressed: darken(hex, 0.15),
  }
  setAccentColor(color)
}

function isValidHex(hex: string): boolean {
  return /^#[0-9a-fA-F]{6}$/.test(hex)
}

initTheme()
initAccentColor()

watchEffect(() => {
  document.documentElement.classList.toggle('dark', isDark.value)
  syncTauriTheme(isDark.value)
})

watchEffect(() => {
  const el = document.documentElement
  el.style.setProperty('--color-accent', accentColor.value.primary)
  el.style.setProperty('--color-accent-hover', accentColor.value.hover)
  el.style.setProperty('--color-accent-pressed', accentColor.value.pressed)
})

export function useTheme() {
  return {
    isDark,
    toggleTheme,
    accentColor,
    setAccentColor,
    setCustomAccentColor,
    isPresetSelected,
    isValidHex,
    isPreset,
    PRESET_COLORS,
  }
}
