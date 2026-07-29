import { motion } from 'motion/react'
import { AlertTriangle, RefreshCw, X } from 'lucide-react'

interface SidecarErrorScreenProps {
  message: string
  onRetry: () => void
  onDismiss?: () => void
}

export function SidecarErrorScreen({ message, onRetry, onDismiss }: SidecarErrorScreenProps) {
  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/95 backdrop-blur-sm"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
    >
      <motion.div
        className="max-w-md w-full rounded-2xl border bg-card p-6 shadow-xl text-center"
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: -20 }}
        transition={{ duration: 0.2 }}
      >
        <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
          <AlertTriangle className="h-6 w-6 text-destructive" />
        </div>

        <h2 className="mt-4 text-lg font-semibold">خطای موتور Aether</h2>

        <p className="mt-2 text-sm text-muted-foreground">
          موتور اتصال با خطا مواجه شده است. ممکن است فایل aether.exe یافت نشده باشد یا مسیر شبکه بسته شده باشد.
        </p>

        <div className="mt-4 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-left">
          <p className="text-xs font-mono text-destructive whitespace-pre-wrap">{message}</p>
        </div>

        <div className="mt-6 flex flex-col gap-3">
          <motion.button
            onClick={onRetry}
            whileTap={{ scale: 0.98 }}
            className="w-full py-2.5 px-4 rounded-lg bg-primary text-primary-foreground font-medium hover:bg-primary/90 transition-colors"
          >
            <RefreshCw className="mr-2 h-4 w-4 inline" />
            تلاش مجدد
          </motion.button>

          {onDismiss && (
            <motion.button
              onClick={onDismiss}
              whileTap={{ scale: 0.98 }}
              className="w-full py-2.5 px-4 rounded-lg border border-border bg-background text-foreground font-medium hover:bg-accent transition-colors"
            >
              <X className="mr-2 h-4 w-4 inline" />
              نادیده بگیر
            </motion.button>
          )}
        </div>
      </motion.div>
    </motion.div>
  )
}