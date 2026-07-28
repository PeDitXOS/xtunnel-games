import { motion } from 'motion/react'
import { cn } from '@/lib/utils'
import { Loader2, Check, AlertTriangle, Wifi, WifiOff } from 'lucide-react'
import { useAppStore } from '@/store/appStore'

function formatDuration(ms: number): string {
  const total = Math.floor(ms / 1000)
  const h = String(Math.floor(total / 3600)).padStart(2, '0')
  const m = String(Math.floor((total % 3600) / 60)).padStart(2, '0')
  const s = String(total % 60).padStart(2, '0')
  return `${h}:${m}:${s}`
}

const MESSAGES: Record<string, string> = {
  idle: 'قطع شده',
  launching: 'شروع Aether...',
  connecting: 'پیدا کردن مسیر...',
  connected: 'متصل',
  reconnecting: 'باز اتصال...',
  disconnecting: 'قطع اتصال...',
  error: 'خطا',
}

export function StatusDisplay() {
  const { status } = useAppStore()
  const phase = status.state

  return (
    <div className="flex flex-col items-center gap-3 w-full">
      <motion.div
        className="relative flex h-28 w-28 items-center justify-center rounded-full"
        animate={{ scale: status.state === 'connected' ? 1 : 1 }}
        transition={{ duration: 0.3 }}
      >
        <div className="absolute inset-0 rounded-full bg-background" />
        <div className={cn('absolute inset-0 rounded-full', {
          'animate-pulse': ['launching', 'connecting', 'reconnecting'].includes(status.state),
          'animate-ping': status.state === 'connected',
          'animate-spin': ['launching', 'connecting', 'reconnecting'].includes(status.state),
        })}>
          <div className="absolute inset-0 rounded-full bg-current/10" />
        </div>

        <div className={cn('relative flex h-full w-full items-center justify-center')}>
          {status.state === 'connected' && (
            <motion.div
              className="absolute inset-0 rounded-full border-2 border-green-500"
              initial={{ scale: 0.85, opacity: 0.5 }}
              animate={{ scale: 2, opacity: 0 }}
              transition={{ duration: 1.5, repeat: Infinity, ease: 'easeOut' }}
            />
          )}
          {status.state === 'idle' && <WifiOff className="h-10 w-10 text-muted-foreground" />}
          {['launching', 'connecting', 'reconnecting'].includes(status.state) && (
            <Loader2 className="h-10 w-10 animate-spin text-primary" />
          )}
          {status.state === 'connected' && <Check className="h-10 w-10 text-green-500" />}
          {status.state === 'error' && <AlertTriangle className="h-10 w-10 text-destructive" />}
        </div>
      </motion.div>

      <div className="text-center w-full max-w-xs">
        <motion.h3
          key={status.state}
          className={cn('text-lg font-medium', {
            'text-muted-foreground': status.state === 'idle',
            'text-primary': ['launching', 'connecting', 'reconnecting'].includes(status.state),
            'text-green-500': status.state === 'connected',
            'text-yellow-500': status.state === 'reconnecting',
            'text-destructive': status.state === 'error',
          })}
          initial={{ y: 4, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          exit={{ y: -4, opacity: 0 }}
          transition={{ duration: 0.1, ease: [0.4, 0, 0.2, 1] }}
        >
          {MESSAGES[status.state] || status.message}
        </motion.h3>

        {status.state === 'connected' && status.connectedAt && (
          <motion.p
            className="text-sm font-mono text-muted-foreground"
            initial={{ y: 4, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: -4, opacity: 0 }}
          >
            {formatDuration(Date.now() - status.connectedAt)}
          </motion.p>
        )}

        {status.state === 'connecting' && status.message !== 'پیدا کردن مسیر...' && (
          <motion.p
            className="text-xs text-muted-foreground"
            initial={{ y: 4, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
          >
            {status.message}
          </motion.p>
        )}

        {status.state === 'error' && (
          <motion.p
            className="text-xs text-destructive"
            initial={{ y: 4, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
          >
            {status.message}
          </motion.p>
        )}
      </div>
    </div>
  )
}