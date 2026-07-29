import { useState, useEffect } from 'react'
import { motion } from 'motion/react'
import { cn } from '@/lib/utils'
import { Download, CheckCircle, RotateCcw, ExternalLink } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'

interface UpdateInfo {
  version: string
  current_version: string
  has_update: boolean
  notes: string
  download_url: string
  published_at: string
}

export function UpdateBanner() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null)
  const [checking, setChecking] = useState(false)

  const checkForUpdates = async () => {
    setChecking(true)
    try {
      const info = await invoke<UpdateInfo>('check_updates')
      setUpdateInfo(info)
    } catch (e) {
      console.error('Update check failed:', e)
    } finally {
      setChecking(false)
    }
  }

  useEffect(() => {
    checkForUpdates()
  }, [])

  if (!updateInfo || !updateInfo.has_update) {
    return (
      <button
        onClick={checkForUpdates}
        disabled={checking}
        className={cn(
          'fixed bottom-4 right-4 z-40 flex items-center gap-2 px-3 py-2 rounded-lg bg-card border border-border shadow-lg',
          'hover:bg-accent transition-colors'
        )}
      >
        <RotateCcw className={cn('h-4 w-4', checking && 'animate-spin')} />
        <span className="text-xs">بررسی آپدیت</span>
      </button>
    )
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="fixed bottom-4 right-4 left-4 z-50 max-w-md"
    >
      <div className="bg-card border border-primary/30 shadow-xl rounded-xl overflow-hidden">
        <div className="p-4 bg-primary/10 border-b border-primary/20 flex items-center gap-3">
          <div className="flex-1">
            <h4 className="font-semibold text-primary">آپدیت موجود است</h4>
            <p className="text-sm text-muted-foreground">
              نسخه {updateInfo.version} منتشر شده
            </p>
          </div>
          <CheckCircle className="h-5 w-5 text-green-500" />
        </div>

        <div className="p-4 space-y-3">
          <div className="text-sm text-muted-foreground bg-muted/50 p-3 rounded-lg max-h-32 overflow-y-auto">
            {updateInfo.notes || 'تغییرات مشخص نشده'}
          </div>

          <div className="flex gap-2">
            <a
              href={updateInfo.download_url}
              target="_blank"
              rel="noreferrer"
              className="flex-1 py-2.5 px-4 bg-primary text-primary-foreground rounded-lg font-medium hover:bg-primary/90 transition-colors flex items-center justify-center gap-2"
            >
              <Download className="h-4 w-4" />
              دانلود
            </a>

            <a
              href={`https://github.com/PeDitXOS/xtunnel-games/releases`}
              target="_blank"
              rel="noreferrer"
              className="px-4 py-2.5 border border-border bg-background text-foreground rounded-lg font-medium hover:bg-accent transition-colors flex items-center justify-center gap-2"
            >
              <ExternalLink className="h-4 w-4" />
              صفحه ریلیز
            </a>
          </div>
        </div>
      </div>
    </motion.div>
  )
}
