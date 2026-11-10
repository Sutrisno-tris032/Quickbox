import React from 'react'
import {
  Globe,
  Moon,
  Sun,
} from 'lucide-react'

export interface TopNavbarProps {
  darkMode: boolean
  onToggleDarkMode: () => void
  currentDomain: string
}

export const TopNavbar: React.FC<TopNavbarProps> = ({
  darkMode,
  onToggleDarkMode,
  currentDomain,
}) => {
  return (
    <div className="h-full px-4 flex items-center justify-between">
      {/* Brand & Sync Status */}
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2.5">
          <img
            src="/quickbox-icon.svg"
            alt="Quick-Box Logo"
            className="w-7 h-7 rounded-lg shrink-0 object-contain"
          />
          <span className="font-bold text-sm tracking-tight text-slate-900 dark:text-white flex items-center gap-1.5">
            Quick-Box
          </span>
        </div>

        <div className="hidden sm:flex items-center gap-2 pl-3 border-l border-slate-200 dark:border-slate-800 text-[11px] text-slate-500 dark:text-slate-400">
          <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
          <span className="font-medium text-emerald-600 dark:text-emerald-400">
            CONNECTED / AUTO-SYNC ON
          </span>
        </div>
      </div>

      {/* Center Nav Pills */}
      <div className="hidden md:flex items-center gap-1 bg-slate-100 dark:bg-slate-800/80 p-1 rounded-xl text-xs font-semibold">
        <button className="px-3 py-1.5 rounded-lg bg-white dark:bg-slate-900 text-blue-600 dark:text-blue-400 shadow-xs cursor-pointer">
          Live Inbox
        </button>
      </div>

      {/* Right Actions */}
      <div className="flex items-center gap-2">
        <div className="hidden lg:flex items-center gap-1.5 px-2.5 py-1 rounded-lg border border-slate-200 dark:border-slate-800 text-[11px] font-mono text-slate-600 dark:text-slate-400 bg-slate-50 dark:bg-slate-900/50">
          <Globe className="w-3.5 h-3.5 text-blue-500" />
          <span>{currentDomain}</span>
        </div>

        <button
          onClick={onToggleDarkMode}
          className="p-1.5 rounded-lg border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition cursor-pointer"
          title="Ganti Tema"
        >
          {darkMode ? <Sun className="w-4 h-4 text-amber-400" /> : <Moon className="w-4 h-4" />}
        </button>
      </div>
    </div>
  )
}
