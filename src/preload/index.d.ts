import type { NekoApi } from '../shared/ipc'
import type { ElectronAPI } from '@electron-toolkit/preload'

declare global {
  interface Window {
    electron: ElectronAPI
    neko: NekoApi
  }
}

export {}
