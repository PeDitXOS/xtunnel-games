export const PROTOCOL_OPTIONS = [
  { value: 'auto', label: 'خودکار (پیشنهادی)' },
  { value: 'masque', label: 'MASQUE (HTTP/3)' },
  { value: 'wireguard', label: 'WireGuard' },
  { value: 'gool', label: 'WARP-in-WARP (gool)' },
] as const

export const SCAN_MODE_OPTIONS = [
  { value: 'turbo', label: 'Turbo (سریع‌ترین)' },
  { value: 'balanced', label: 'Balanced (متعادل)' },
  { value: 'thorough', label: 'Thorough (دقیق)' },
  { value: 'stealth', label: 'Stealth (مخفی‌ترین)' },
] as const

export const IP_VERSION_OPTIONS = [
  { value: 'v4', label: 'IPv4' },
  { value: 'v6', label: 'IPv6' },
  { value: 'both', label: 'هر دو' },
] as const

export const CATEGORIES = [
  { id: 'all', label: 'همه', icon: 'Globe' },
  { id: 'games', label: 'بازی‌ها', icon: 'Gamepad2' },
  { id: 'launchers', label: 'لانچرها', icon: 'Download' },
  { id: 'media', label: 'یوتیوب/دسکورد', icon: 'MessageSquare' },
  { id: 'other', label: 'سایر', icon: 'Zap' },
] as const

export type Category = typeof CATEGORIES[number]['id']