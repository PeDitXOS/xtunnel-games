'use client'

import { useState } from 'react'
import { cn } from '@/lib/utils'
import { ChevronDown, Settings2, Info } from 'lucide-react'
import { useAppStore } from '@/store/appStore'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '@/components/ui/tooltip'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { PROTOCOL_OPTIONS, SCAN_MODE_OPTIONS, IP_VERSION_OPTIONS } from './constants'

export function AdvancedPanel() {
  const { config, setConfig, status, logs } = useAppStore()
  const [open, setOpen] = useState(false)

  const locked = status.state !== 'idle' && status.state !== 'error'

  return (
    <TooltipProvider>
      <Collapsible open={open} onOpenChange={setOpen}>
        <CollapsibleTrigger className="w-full flex items-center justify-center gap-2 py-2 text-sm text-muted-foreground hover:text-foreground rounded-md">
          <Settings2 className="h-4 w-4" />
          تنظیمات پیشرفته
          <ChevronDown
            className={cn(
              'h-4 w-4 transition-transform duration-150',
              open ? 'rotate-180' : ''
            )}
          />
        </CollapsibleTrigger>

        <CollapsibleContent className="overflow-hidden data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:slide-in-from-bottom-2 data-[state=open]:duration-200">
          <div className="space-y-4 pb-2">
            <div className="space-y-2">
              <label className="flex items-center gap-2 text-xs text-muted-foreground">
                پروتکل
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Info className="h-3.5 w-3.5" />
                  </TooltipTrigger>
                  <TooltipContent>
                    MASQUE ترافیک را HTTPS نشان می‌دهد. WireGuard سبک‌تر. gool دو تونل تودرتو.
                  </TooltipContent>
                </Tooltip>
              </label>
              <Select value={config.protocol} onValueChange={(v: string) => setConfig({ protocol: v as any })} disabled={locked}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="انتخاب پروتکل" />
                </SelectTrigger>
                <SelectContent>
                  {PROTOCOL_OPTIONS.map(opt => (
                    <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <label className="flex items-center gap-2 text-xs text-muted-foreground">
                حالت اسکن
                <Tooltip>
                  <TooltipTrigger asChild><Info className="h-3.5 w-3.5" /></TooltipTrigger>
                  <TooltipContent>
                    Turbo سریع‌ترین. Balanced پیش‌فرض. Thorough دقیق‌تر. Stealth مخفی‌ترین.
                  </TooltipContent>
                </Tooltip>
              </label>
              <Select value={config.scanMode} onValueChange={(v: string) => setConfig({ scanMode: v as any })} disabled={locked}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="انتخاب حالت" />
                </SelectTrigger>
                <SelectContent>
                  {SCAN_MODE_OPTIONS.map(opt => (
                    <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <label className="flex items-center gap-2 text-xs text-muted-foreground">
                نسخه IP
                <Tooltip>
                  <TooltipTrigger asChild><Info className="h-3.5 w-3.5" /></TooltipTrigger>
                  <TooltipContent>
                    IPv4 پیش‌فرض امن است.
                  </TooltipContent>
                </Tooltip>
              </label>
              <Select value={config.ipVersion} onValueChange={(v: string) => setConfig({ ipVersion: v as any })} disabled={locked}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="انتخاب نسخه IP" />
                </SelectTrigger>
                <SelectContent>
                  {IP_VERSION_OPTIONS.map(opt => (
                    <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between pt-2 border-t">
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                اتصال سریع
                <Tooltip>
                  <TooltipTrigger asChild><Info className="h-3.5 w-3.5" /></TooltipTrigger>
                  <TooltipContent>
                    آخرین گیت‌وی را ذخیره و در اتصال بعدی ابتدا تست می‌کند.
                  </TooltipContent>
                </Tooltip>
              </div>
              <Switch
                checked={config.quickReconnect}
                onCheckedChange={(v: boolean) => setConfig({ quickReconnect: v })}
                disabled={locked}
              />
            </div>

            <div className="space-y-2 pt-2 border-t">
              <div className="flex items-center gap-2">
                <div className="h-px flex-1 bg-border" />
                <span className="text-[10px] tracking-widest text-muted-foreground uppercase">لاگ‌ها</span>
                <div className="h-px flex-1 bg-border" />
              </div>

              <div className="max-h-64 overflow-y-auto rounded-md bg-black/20 p-2 font-mono text-xs text-muted-foreground border border-white/10">
                {logs.length === 0 ? (
                  <p className="text-muted-foreground">هیچ خروجی‌ای هنوز وجود ندارد.</p>
                ) : (
                  logs.map((l, i) => (
                    <p key={i} className="whitespace-pre-wrap font-mono">{l}</p>
                  ))
                )}
              </div>
            </div>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </TooltipProvider>
  )
}
