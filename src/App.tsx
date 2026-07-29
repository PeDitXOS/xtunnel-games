import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'motion/react'
import { Gamepad2, Globe, MessageSquare, Youtube, FolderOpen, CheckCircle2, Search, Zap, RefreshCw } from 'lucide-react'
import { useAppStore } from './store/appStore'
import { ConnectButton } from './components/ConnectButton'
import { StatusDisplay } from './components/StatusDisplay'
import { SettingsPanel } from './components/SettingsPanel'
import { TitleBar } from './components/TitleBar'
import { SidecarErrorScreen } from './components/SidecarErrorScreen'
import { cn } from './lib/utils'

const SCREEN_TRANSITION = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -4 },
  transition: { duration: 0.16, ease: [0.22, 1, 0.36, 1] as const },
}

function App() {
  const { sidecarError, retryAfterSidecarError, connect, scanApps, apps, selectedApps, toggleApp, initialize } = useAppStore()

  useEffect(() => {
    initialize()
  }, [initialize])

  return (
    <div className="relative flex h-svh w-full flex-col overflow-hidden bg-background">
      <TitleBar />
      <div className="relative min-h-0 flex-1">
        <AnimatePresence mode="sync">
          {sidecarError ? (
            <motion.div
              key="error"
              className="absolute inset-0 z-10 flex items-center justify-center p-6"
              {...SCREEN_TRANSITION}
            >
              <SidecarErrorScreen
                message={sidecarError}
                onRetry={() => {
                  retryAfterSidecarError()
                  connect()
                }}
              />
            </motion.div>
          ) : (
            <motion.div
              key="main"
              className="absolute inset-0 flex flex-col"
              {...SCREEN_TRANSITION}
            >
              <main className="flex-1 flex flex-col items-center p-6 overflow-y-auto">
                <div className="w-full max-w-2xl flex flex-col items-center gap-6">
                  <StatusDisplay />
                  <ConnectButton />
                </div>
                <SettingsPanel />
              </main>
              <aside className="w-full max-w-2xl px-6 pb-6">
                <AppListSection apps={apps} selectedApps={selectedApps} onToggle={toggleApp} onScan={scanApps} />
              </aside>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  )
}

interface AppListSectionProps {
  apps: Array<{ exeName: string; name: string; category: string }>
  selectedApps: string[]
  onToggle: (exeName: string) => void
  onScan: () => void
}

function AppListSection({ apps, selectedApps, onToggle, onScan }: AppListSectionProps) {
  const [search, setSearch] = useState('')
  const [filter, setFilter] = useState<'all' | 'games' | 'launchers' | 'media' | 'social' | 'other'>('all')
  const [showOnlySelected, setShowOnlySelected] = useState(false)

  const filteredApps = apps
    .filter(app => {
      if (showOnlySelected && !selectedApps.includes(app.exeName)) return false
      if (search && !app.name.toLowerCase().includes(search.toLowerCase()) && !app.exeName.toLowerCase().includes(search.toLowerCase())) return false
      if (filter !== 'all' && app.category !== filter) return false
      return true
    })
    .sort((a, b) => {
      const aSel = selectedApps.includes(a.exeName)
      const bSel = selectedApps.includes(b.exeName)
      if (aSel !== bSel) return bSel ? 1 : -1
      return a.name.localeCompare(b.name, 'fa')
    })

  const categories = [
    { id: 'all', label: 'همه', icon: <Globe className="h-4 w-4" /> },
    { id: 'games', label: 'بازی‌ها', icon: <Gamepad2 className="h-4 w-4" /> },
    { id: 'launchers', label: 'لانچرها', icon: <FolderOpen className="h-4 w-4" /> },
    { id: 'media', label: 'مدیا', icon: <Youtube className="h-4 w-4" /> },
    { id: 'social', label: 'اجتماعی', icon: <MessageSquare className="h-4 w-4" /> },
    { id: 'other', label: 'سایر', icon: <Zap className="h-4 w-4" /> },
  ] as const

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold flex items-center gap-2">
              <Gamepad2 className="h-5 w-5" />
              برنامه‌ها برای تونلینگ
            </h3>
            <p className="text-sm text-muted-foreground">
              {selectedApps.length} انتخاب شده از {apps.length} برنامه
            </p>
          </div>
          <button
            onClick={onScan}
            disabled={useAppStore.getState().scanning}
            className="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 flex items-center gap-1"
          >
            <RefreshCw className="h-4 w-4" />
            بازاسانی
          </button>
        </div>

        <div className="flex flex-wrap gap-2">
          {categories.map(cat => (
            <button
              key={cat.id}
              onClick={() => setFilter(cat.id)}
              className={cn(
                'flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-full whitespace-nowrap transition-colors',
                filter === cat.id
                  ? 'bg-primary text-primary-foreground shadow-sm'
                  : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
              )}
            >
              {cat.icon}
              {cat.label}
            </button>
          ))}
          <label className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-secondary hover:bg-secondary/80 rounded-full cursor-pointer">
            <input
              type="checkbox"
              checked={showOnlySelected}
              onChange={e => setShowOnlySelected(e.target.checked)}
              className="h-4 w-4 rounded border-border"
            />
            <CheckCircle2 className="h-4 w-4" />
            فقط انتخاب‌شده‌ها
          </label>
        </div>

        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="جستجوی برنامه..."
            value={search}
            onChange={e => setSearch(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-input border border-border rounded-lg text-sm placeholder:text-muted-foreground focus:ring-2 focus:ring-primary focus:border-transparent"
            dir="ltr"
          />
        </div>
      </div>

      <div className="max-h-96 overflow-y-auto space-y-2 scrollbar-hide">
        {apps.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground">
            <FolderOpen className="h-12 w-12 mb-4 opacity-50" />
            <p className="text-lg">برنامه‌ای یافت نشد</p>
            <p className="text-sm">دکمه بازاسانی را بزنید</p>
            <button
              onClick={onScan}
              className="mt-4 px-4 py-2 bg-primary text-primary-foreground rounded-lg"
            >
              اسکن برنامه‌ها
            </button>
          </div>
        ) : filteredApps.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground">
            <Search className="h-12 w-12 mb-4 opacity-50" />
            <p>برنامه‌ای با این فیلتر یافت نشد</p>
          </div>
        ) : (
          filteredApps.map(app => (
            <AppItem
              key={app.exeName}
              app={app}
              selected={selectedApps.includes(app.exeName)}
              onToggle={() => onToggle(app.exeName)}
            />
          ))
        )}
      </div>

      {selectedApps.length > 0 && (
        <div className="p-3 bg-primary/10 border border-primary/20 rounded-lg">
          <p className="text-sm font-medium text-primary">
            {selectedApps.length} برنامه برای تونلینگ انتخاب شده
          </p>
          <p className="text-xs text-primary/80 mt-1">
            ترافیک این برنامه‌ها از تونل Aether عبور می‌کند. بقیه مستقیم هستند.
          </p>
        </div>
      )}
    </div>
  )
}

interface AppItemProps {
  app: {
    exeName: string
    name: string
    category: string
    iconPath?: string
  }
  selected: boolean
  onToggle: (exeName: string) => void
}

function AppItem({ app, selected, onToggle }: AppItemProps) {
  const CATEGORY_LABELS: Record<string, string> = {
    games: 'بازی',
    launchers: 'لانچر',
    media: 'مدیا',
    social: 'اجتماعی',
    other: 'سایر',
  }

  return (
    <motion.div
      whileHover={{ y: -1, boxShadow: '0 4px 12px rgba(0,0,0,0.15)' }}
      onClick={() => onToggle(app.exeName)}
      className={cn(
        'relative flex items-center gap-3 p-3 rounded-xl border transition-all',
        selected
          ? 'border-primary/50 bg-primary/5 ring-1 ring-primary/20'
          : 'border-border bg-card/50 hover:bg-card hover:border-border/50'
      )}
    >
      <AnimatePresence mode="wait">
        {selected && (
          <motion.div
            key="check"
            initial={{ scale: 0, rotate: -180 }}
            animate={{ scale: 1, rotate: 0 }}
            exit={{ scale: 0, rotate: 180 }}
            transition={{ duration: 0.2, type: 'spring', stiffness: 300, damping: 20 }}
            className="absolute -top-1 -right-1 flex h-6 w-6 items-center justify-center rounded-full bg-primary text-primary-foreground"
          >
            <CheckCircle2 className="h-4 w-4" />
          </motion.div>
        )}
      </AnimatePresence>

      {app.iconPath && (
        <img
          src={app.iconPath}
          alt={app.name}
          className="h-10 w-10 rounded-lg object-cover flex-shrink-0"
          onError={e => { e.currentTarget.style.display = 'none' }}
        />
      )}

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <h4 className="font-medium truncate">{app.name}</h4>
          <span className={cn(
            'px-2 py-0.5 text-xs rounded-full',
            selected ? 'bg-primary/20 text-primary' : 'bg-muted text-muted-foreground'
          )}>
            {CATEGORY_LABELS[app.category]}
          </span>
        </div>
        <p className="text-xs text-muted-foreground truncate font-mono">{app.exeName}</p>
      </div>

      <motion.div
        initial={{ scale: 0, rotate: -180 }}
        animate={{ scale: 1, rotate: 0 }}
        exit={{ scale: 0, rotate: 180 }}
        transition={{ duration: 0.2, type: 'spring', stiffness: 300, damping: 20 }}
        className={cn(
          'flex h-6 w-6 items-center justify-center rounded-lg',
          selected ? 'bg-primary/20 text-primary' : 'bg-muted text-muted-foreground hover:bg-muted/50'
        )}
      >
        <CheckCircle2 className="h-4 w-4" />
      </motion.div>
    </motion.div>
  )
}

export default App