import { useState, useEffect } from 'react'
import { motion } from 'motion/react'
import { cn } from '@/lib/utils'
import { Download, AlertCircle, CheckCircle, RotateCcw, ExternalLink } from 'lucide-react'
import { useAppStore } from '@/store/appStore'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '@/components/ui/tooltip'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface UpdateInfo {
  version: string
  currentVersion: string
  hasUpdate: boolean
  notes: string
  downloadUrl: string
  publishedAt: string
}

export function UpdateBanner() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null)
  const [checking, setChecking] = useState(false)
  const { sidecarError, retryAfterSidecarError } = useAppStore()

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
    
    // Listen for updater events
    const unlisten = await listen('tauri://update', (event: any) => {
      if (event.payload.status === 'pending') {
        checkForUpdates()
      }
    })
    return () => unlisten()
  }, [])

  if (!updateInfo || !updateInfo.hasUpdate) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
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
          </TooltipTrigger>
          <TooltipContent>نسخه فعلی: v0.1.0</TooltipContent>
        </Tooltip>
      </TooltipProvider>
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
          <AlertCircle className="h-5 w-5 text-primary" />
          <div className="flex-1">
            <h4 className="font-semibold text-primary">آپدیت موجود است</h4>
            <p className="text-sm text-muted-foreground">
              نسخه {updateInfo.version} منتشر شده (شما: v{updateInfo.currentVersion})
            </p>
          </div>
          <CheckCircle className="h-5 w-5 text-green-500" />
        </div>

        <div className="p-4 space-y-3">
          <div className="text-sm text-muted-foreground bg-muted/50 p-3 rounded-lg max-h-32 overflow-y-auto">
            {updateInfo.notes || 'تغییرات مشخص نشده'}
          </div>

          <div className="flex gap-2">
            <motion.button
              onClick={() => invoke('install_update')}
              whileTap={{ scale: 0.98 }}
              className="flex-1 py-2.5 px-4 bg-primary text-primary-foreground rounded-lg font-medium hover:bg-primary/90 transition-colors flex items-center justify-center gap-2"
            >
              <Download className="h-4 w-4" />
              دانلود و نصب
            </motion.button>

            <motion.button
              onClick={() => window.open(updateInfo.downloadUrl, '_blank')}
              whileTap={{ scale: 0.98 }}
              className="px-4 py-2.5 border border-border bg-background text-foreground rounded-lg font-medium hover:bg-accent transition-colors flex items-center justify-center gap-2"
            >
              <ExternalLink className="h-4 w-4" />
              صفحه ریلیز
            </motion.button>
          </div>

          <p className="text-xs text-muted-foreground text-center">
            منتشر شده: {new Date(updateInfo.publishedAt).toLocaleDateString('fa-IR')}
          </p>
        </div>
      </div>
    </motion.div>
  )
}