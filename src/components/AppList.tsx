'use client'

import { useState } from 'react'
import { motion } from 'motion/react'
import { cn } from '@/lib/utils'
import { Search, Gamepad2, Globe, MessageSquare, Youtube, FolderOpen, CheckCircle2 } from 'lucide-react'
import { useAppStore } from '@/store/appStore'

type Category = 'all' | 'games' | 'launchers' | 'media' | 'social' | 'other'

const CATEGORIES: { id: Category; label: string; icon: React.ReactNode }[] = [
  { id: 'all', label: 'همه', icon: <Globe className="h-4 w-4" /> },
  { id: 'games', label: 'بازی‌ها', icon: <Gamepad2 className="h-4 w-4" /> },
  { id: 'launchers', label: 'لانچرها', icon: <FolderOpen className="h-4 w-4" /> },
  { id: 'media', label: 'مدیا', icon: <Youtube className="h-4 w-4" /> },
  { id: 'social', label: 'شبکه‌های اجتماعی', icon: <MessageSquare className="h-4 w-4" /> },
  { id: 'other', label: 'سایر', icon: <Globe className="h-4 w-4" /> },
]

export function AppList() {
  const { apps, selectedApps, scanning, scanApps, toggleApp } = useAppStore()
  const [category, setCategory] = useState<Category>('all')
  const [search, setSearch] = useState('')

  const filteredApps = apps
    .filter(app => {
      if (category !== 'all' && app.category !== category) return false
      if (search && !app.name.toLowerCase().includes(search.toLowerCase()) && !app.exeName.toLowerCase().includes(search.toLowerCase())) return false
      return true
    })
    .sort((a, b) => {
      if (selectedApps.includes(a.exeName) !== selectedApps.includes(b.exeName)) {
        return selectedApps.includes(b.exeName) ? 1 : -1
      }
      return a.name.localeCompare(b.name)
    })

  const groupedApps = filteredApps.reduce((acc, app) => {
    const cat = app.category
    if (!acc[cat]) acc[cat] = []
    acc[cat].push(app)
    return acc
  }, {} as Record<string, typeof apps>)

  const selectedCount = selectedApps.length

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between border-b p-4 bg-card/50">
        <div>
          <h2 className="text-lg font-semibold">برنامه‌ها</h2>
          <p className="text-sm text-muted-foreground">
            {selectedCount} انتخاب شده از {apps.length} برنامه
          </p>
        </div>
        <motion.button
          onClick={scanApps}
          disabled={scanning}
          whileTap={{ scale: 0.97 }}
          className="flex items-center gap-2 px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50"
        >
          <Search className="h-4 w-4" />
          بازاسانی
        </motion.button>
      </div>

      {/* Search & Filter */}
      <div className="p-4 border-b space-y-3 bg-card/30">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="جستجوی برنامه..."
            value={search}
            onChange={e => setSearch(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-input border border-border rounded-lg text-sm placeholder:text-muted-foreground focus:ring-2 focus:ring-primary focus:border-transparent"
          />
        </div>

        <div className="flex gap-2 overflow-x-auto pb-2">
          {CATEGORIES.map(cat => (
            <motion.button
              key={cat.id}
              onClick={() => setCategory(cat.id)}
              whileTap={{ scale: 0.97 }}
              className={cn(
                'flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-full whitespace-nowrap transition-colors',
                category === cat.id
                  ? 'bg-primary text-primary-foreground shadow-sm'
                  : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
              )}
            >
              {cat.icon}
              {cat.label}
            </motion.button>
          ))}
        </div>
      </div>

      {/* App List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {apps.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground">
            <FolderOpen className="h-12 w-12 mb-4 opacity-50" />
            <p className="text-lg">برنامه‌ای یافت نشد</p>
            <p className="text-sm">دکمه بازاسانی را بزنید</p>
            <motion.button
              onClick={scanApps}
              whileTap={{ scale: 0.97 }}
              className="mt-4 px-4 py-2 bg-primary text-primary-foreground rounded-lg"
            >
              اسکن برنامه‌ها
            </motion.button>
          </div>
        ) : filteredApps.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground">
            <Search className="h-12 w-12 mb-4 opacity-50" />
            <p>برنامه‌ای با این فیلتر یافت نشد</p>
          </div>
        ) : (
          <>
            {Object.entries(groupedApps).map(([cat, apps]) => (
              <motion.div
                key={cat}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.2 }}
                className="space-y-2"
              >
                <div className="flex items-center gap-2 px-2 py-1 text-xs font-medium text-muted-foreground uppercase tracking-wider">
                  {CATEGORIES.find(c => c.id === cat as Category)?.icon}
                  {CATEGORIES.find(c => c.id === cat as Category)?.label}
                  <span className="ml-auto px-2 py-0.5 bg-secondary rounded-full text-xs">
                    {apps.length}
                  </span>
                </div>
                {apps.map(app => (
                  <AppItem
                    key={app.exeName}
                    app={app}
                    selected={selectedApps.includes(app.exeName)}
                    onToggle={toggleApp}
                  />
                ))}
              </motion.div>
            ))}
          </>
        )}
      </div>

      {/* Footer */}
      <div className="border-t p-4 bg-card/50">
        <motion.button
          onClick={() => useAppStore.getState().connect()}
          disabled={selectedCount === 0}
          whileTap={{ scale: 0.98 }}
          className={cn(
            'w-full py-3 rounded-lg font-medium text-lg transition-colors',
            selectedCount === 0
              ? 'bg-muted text-muted-foreground cursor-not-allowed'
              : 'bg-primary text-primary-foreground hover:bg-primary/90'
          )}
        >
          {selectedCount === 0 ? 'حداقل یک برنامه انتخاب کنید' : `اتصال و روت کردن ${selectedCount} برنامه`}
        </motion.button>
      </div>
    </div>
  )
}

interface AppItemProps {
  app: AppInfo
  selected: boolean
  onToggle: (exeName: string) => void
}

function AppItem({ app, selected, onToggle }: AppItemProps) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 20 }}
      transition={{ duration: 0.15 }}
      className={cn(
        'relative flex items-center gap-3 px-3 py-2.5 rounded-xl bg-card border transition-all cursor-pointer',
        selected
          ? 'border-primary/50 bg-primary/5 ring-1 ring-primary/20'
          : 'border-border hover:border-primary/30'
      )}
      onClick={() => onToggle(app.exeName)}
    >
      <div className={cn(
        'flex h-10 w-10 items-center justify-center rounded-lg flex-shrink-0',
        selected ? 'bg-primary text-primary-foreground' : 'bg-secondary text-secondary-foreground'
      )}>
        {selected ? <CheckCircle2 className="h-5 w-5" /> : <FolderOpen className="h-5 w-5" />}
      </div>

      <div className="flex-1 min-w-0">
        <p className="font-medium truncate">{app.name}</p>
        <p className="text-xs text-muted-foreground truncate">{app.exeName}</p>
      </div>

      <motion.button
        onClick={(e) => { e.stopPropagation(); onToggle(app.exeName); }}
        whileTap={{ scale: 0.9 }}
        className={cn(
          'flex h-9 w-9 items-center justify-center rounded-full transition-colors',
          selected
            ? 'bg-primary/10 text-primary hover:bg-primary/20'
            : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
        )}
        aria-label={selected ? 'حذف از انتخاب' : 'افزودن به انتخاب'}
      >
        <CheckCircle2 className="h-5 w-5" />
      </motion.button>
    </motion.div>
  )
}

// Types
interface AppInfo {
  name: string
  exeName: string
  exePath: string
  iconPath?: string
  category: string
}
