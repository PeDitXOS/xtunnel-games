import { createRoot } from 'react-dom/client'
import { TooltipProvider } from '@/components/ui/tooltip'
import { Toaster } from '@/components/ui/toaster'
import App from './App'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <TooltipProvider>
    <App />
    <Toaster />
  </TooltipProvider>
)
