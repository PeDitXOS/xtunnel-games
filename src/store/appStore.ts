import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

export interface AppInfo {
  name: string
  exeName: string
  exePath: string
  iconPath?: string
  category: 'games' | 'launchers' | 'media' | 'social' | 'other'
}

export interface ConnectionStatus {
  state: 'idle' | 'launching' | 'connecting' | 'connected' | 'reconnecting' | 'disconnecting' | 'error'
  message: string
  socksPort?: number
  connectedAt?: number
  protocol?: string
  scanMode?: string
}

export interface AetherConfig {
  protocol: 'auto' | 'masque' | 'wireguard' | 'gool'
  scanMode: 'turbo' | 'balanced' | 'thorough' | 'stealth'
  ipVersion: 'v4' | 'v6' | 'both'
  quickReconnect: boolean
}

interface AppState {
  status: ConnectionStatus
  sidecarError: string | null
  apps: AppInfo[]
  selectedApps: string[]
  config: AetherConfig
  scanning: boolean
  _unlisten: UnlistenFn | null

  initialize: () => Promise<void>
  scanApps: () => Promise<void>
  toggleApp: (exeName: string) => void
  setConfig: (config: Partial<AetherConfig>) => void
  connect: () => Promise<void>
  disconnect: () => Promise<void>
  retryAfterSidecarError: () => void
  _setUnlisten: (fn: UnlistenFn) => void
}

const defaultConfig: AetherConfig = {
  protocol: 'auto',
  scanMode: 'balanced',
  ipVersion: 'v4',
  quickReconnect: true,
}

export const useAppStore = create<AppState>()(
  persist(
    (set, get) => ({
      status: { state: 'idle', message: 'آماده اتصال' },
      sidecarError: null,
      apps: [],
      selectedApps: [],
      config: defaultConfig,
      scanning: false,
      _unlisten: null,

      initialize: async () => {
        try {
          const unlisten = await listen('aether://status', (event: any) => {
            set({ status: event.payload })
          })
          set({ _unlisten: unlisten })
        } catch (e) {
          console.warn('Could not listen to aether events:', e)
        }

        await get().scanApps()
      },

      scanApps: async () => {
        set({ scanning: true })
        try {
          const apps = await invoke<AppInfo[]>('scan_apps')
          set({ apps })
        } catch (e) {
          console.error('Failed to scan apps:', e)
        } finally {
          set({ scanning: false })
        }
      },

      toggleApp: (exeName: string) => {
        const { selectedApps } = get()
        set({
          selectedApps: selectedApps.includes(exeName)
            ? selectedApps.filter(a => a !== exeName)
            : [...selectedApps, exeName]
        })
      },

      setConfig: (config: Partial<AetherConfig>) => {
        set(state => ({
          config: { ...state.config, ...config }
        }))
      },

      connect: async () => {
        const { selectedApps, config } = get()
        if (selectedApps.length === 0) return

        set({ sidecarError: null })
        try {
          await invoke('aether_connect', {
            apps: selectedApps,
            config,
          })
        } catch (e: any) {
          set({ sidecarError: e?.message || String(e) })
        }
      },

      disconnect: async () => {
        try {
          await invoke('aether_disconnect')
        } catch (e) {
          console.error('Disconnect failed:', e)
        }
      },

      retryAfterSidecarError: () => {
        set({ sidecarError: null })
      },

      _setUnlisten: (fn: UnlistenFn) => set({ _unlisten: fn }),
    }),
    {
      name: 'xtunnel-games-store',
      partialize: (state) => ({
        selectedApps: state.selectedApps,
        config: state.config,
      }),
    }
  )
)