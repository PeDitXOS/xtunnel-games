import { AnimatePresence, motion } from 'motion/react'
import { cn } from '@/lib/utils'
import { Loader2, Check, AlertTriangle, Wifi, WifiOff } from 'lucide-react'
import { useAppStore } from '@/store/appStore'

type Phase = 'idle' | 'launching' | 'connecting' | 'connected' | 'reconnecting' | 'disconnecting' | 'error'

const PHASE_CONFIG: Record<Phase, { icon: React.ReactNode; label: string; color: string; anim: string }> = {
  idle: { icon: <WifiOff className="h-10 w-10" />, label: 'اتصال', color: 'text-muted-foreground', anim: '' },
  launching: { icon: <Loader2 className="h-10 w-10 animate-spin" />, label: 'شروع Aether...', color: 'text-primary', anim: 'animate-pulse' },
  connecting: { icon: <Loader2 className="h-10 w-10 animate-spin" />, label: 'پیدا کردن مسیر...', color: 'text-primary', anim: 'animate-pulse' },
  connected: { icon: <Check className="h-10 w-10 text-green-500" />, label: 'متصل', color: 'text-green-500', anim: 'animate-ping' },
  reconnecting: { icon: <Loader2 className="h-10 w-10 animate-spin" />, label: 'باز اتصال...', color: 'text-yellow-500', anim: 'animate-pulse' },
  disconnecting: { icon: <Loader2 className="h-10 w-10 animate-spin" />, label: 'قطع اتصال...', color: 'text-muted-foreground', anim: 'animate-pulse' },
  error: { icon: <AlertTriangle className="h-10 w-10 text-destructive" />, label: 'خطا', color: 'text-destructive', anim: '' },
}

const ARIA_LABELS: Record<Phase, string> = {
  idle: 'اتصال',
  launching: 'لغو شروع',
  connecting: 'لغو اتصال',
  connected: 'قطع اتصال',
  reconnecting: 'لغو باز اتصال',
  disconnecting: 'منتظر بمانید',
  error: 'تلاش مجدد',
}

export function ConnectButton() {
  const { status, connect, disconnect } = useAppStore()
  const phase = status.state
  const config = PHASE_CONFIG[phase]
  const Icon = config.icon as React.ReactNode
  const isTransitioning = phase === 'launching' || phase === 'connecting' || phase === 'reconnecting' || phase === 'disconnecting'

  const handleClick = () => {
    if (phase === 'idle' || phase === 'error') {
      connect()
    } else if (phase !== 'disconnecting') {
      disconnect()
    }
  }

  return (
    <motion.button
      type="button"
      onClick={handleClick}
      disabled={phase === 'disconnecting'}
      whileTap={{ scale: 0.97 }}
      className={cn(
        'relative flex h-32 w-32 items-center justify-center rounded-full',
        'bg-card border-2 border-border',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background',
        'transition-colors'
      )}
      aria-label={ARIA_LABELS[phase]}
    >
      <div className="absolute inset-0 rounded-full bg-background/50" />

      <AnimatePresence mode="wait">
        <motion.div
          key={phase}
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          transition={{ duration: 0.15 }}
          className="relative flex h-full w-full items-center justify-center"
        >
          <div className={cn('absolute inset-0 rounded-full', config.anim)}>
            <div className="absolute inset-0 rounded-full bg-current/10" />
          </div>

          <div className={cn('relative flex h-full w-full items-center justify-center', config.color)}>
            {Icon}
          </div>

          {phase === 'connected' && (
            <motion.div
              className="absolute inset-0 rounded-full border-2 border-green-500"
              initial={{ scale: 0.85, opacity: 0.5 }}
              animate={{ scale: 2, opacity: 0 }}
              transition={{ duration: 1.5, repeat: Infinity, ease: 'easeOut' }}
            />
          )}
        </motion.div>
      </AnimatePresence>

      <AnimatePresence mode="wait">
        <motion.div
          key={phase}
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.8 }}
          transition={{ duration: 0.1, ease: [0.4, 0, 0.2, 1] }}
          className="absolute inset-0 flex items-center justify-center"
        >
          <span className="text-xs font-mono text-muted-foreground/70">
            {config.label}
          </span>
        </motion.div>
      </AnimatePresence>
    </motion.button>
  )
}