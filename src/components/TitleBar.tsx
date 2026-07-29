import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, X, Maximize2 } from 'lucide-react'
import { motion } from 'motion/react'
import { cn } from '@/lib/utils'

export function TitleBar() {
  const win = getCurrentWindow()

  return (
    <div
      data-tauri-drag-region
      className="flex h-10 items-center justify-between border-b border-border bg-background/80 backdrop-blur-sm select-none"
    >
      <div className="flex items-center gap-2 pl-3" data-tauri-drag-region>
        <div className="h-3 w-3 rounded-full bg-primary" />
        <span className="text-xs font-medium text-muted-foreground">Xtunnel Games</span>
      </div>

      <div className="flex items-center">
        <motion.button
          whileHover={{ backgroundColor: 'rgba(255,255,255,0.1)' }}
          whileTap={{ scale: 0.9 }}
          onClick={() => win.minimize()}
          className="flex h-10 w-12 items-center justify-center text-muted-foreground hover:text-foreground"
        >
          <Minus className="h-4 w-4" />
        </motion.button>
        <motion.button
          whileHover={{ backgroundColor: 'rgba(255,255,255,0.1)' }}
          whileTap={{ scale: 0.9 }}
          onClick={() => win.toggleMaximize()}
          className="flex h-10 w-12 items-center justify-center text-muted-foreground hover:text-foreground"
        >
          <Maximize2 className="h-3.5 w-3.5" />
        </motion.button>
        <motion.button
          whileHover={{ backgroundColor: 'rgba(239,68,68,0.8)', color: 'white' }}
          whileTap={{ scale: 0.9 }}
          onClick={() => win.close()}
          className="flex h-10 w-12 items-center justify-center text-muted-foreground hover:text-foreground"
        >
          <X className="h-4 w-4" />
        </motion.button>
      </div>
    </div>
  )
}
