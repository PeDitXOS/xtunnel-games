import { motion } from 'motion/react'
import { cn } from '@/lib/utils'
import { Check, Gamepad2, FolderOpen, Youtube, MessageSquare, Zap } from 'lucide-react'

const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  games: <Gamepad2 className="h-4 w-4" />,
  launchers: <FolderOpen className="h-4 w-4" />,
  media: <Youtube className="h-4 w-4" />,
  social: <MessageSquare className="h-4 w-4" />,
  other: <Zap className="h-4 w-4" />,
}

const CATEGORY_LABELS: Record<string, string> = {
  games: 'بازی',
  launchers: 'لانچر',
  media: 'مدیا',
  social: 'اجتماعی',
  other: 'سایر',
}

interface AppItemProps {
  app: {
    exeName: string
    name: string
    path: string
    iconPath?: string
    category: 'games' | 'launchers' | 'media' | 'social' | 'other'
    pid?: number
  }
  selected: boolean
  onToggle: () => void
}

export function AppItem({ app, selected, onToggle }: AppItemProps) {
  const CategoryIcon = CATEGORY_ICONS[app.category]

  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 20 }}
      transition={{ duration: 0.15 }}
      className={cn(
        'relative flex items-center gap-3 p-3 rounded-xl border transition-all',
        selected
          ? 'border-primary/50 bg-primary/5'
          : 'border-border bg-card/50 hover:bg-card hover:border-border/50'
      )}
      onClick={onToggle}
    >
      <div className={cn(
        'flex h-10 w-10 items-center justify-center rounded-lg flex-shrink-0',
        selected ? 'bg-primary text-primary-foreground' : 'bg-secondary text-secondary-foreground'
      )}>
        {selected ? <Check className="h-5 w-5" /> : CategoryIcon}
      </div>

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
        <Check className="h-4 w-4" />
      </motion.div>
    </motion.div>
  )
}