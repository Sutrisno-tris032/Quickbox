import React, { useState } from 'react'
import { Mail, Clock, Shield, Lock, Unlock, Moon, Sun, ArrowRight } from 'lucide-react'
import { Button } from '../ui/Button'
import type { InboxStatus } from '../../types'

export interface SidebarProps {
  currentInbox: string
  status: InboxStatus | null
  recentInboxes: string[]
  onSelectInbox: (name: string) => void
  onOpenLockModal: () => void
  onRemoveLock: () => void
  darkMode: boolean
  onToggleDarkMode: () => void
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentInbox,
  status,
  recentInboxes,
  onSelectInbox,
  onOpenLockModal,
  onRemoveLock,
  darkMode,
  onToggleDarkMode,
}) => {
  const [newInboxInput, setNewInboxInput] = useState('')

  const handleCreateOrJump = (e: React.FormEvent) => {
    e.preventDefault()
    if (newInboxInput.trim()) {
      onSelectInbox(newInboxInput.trim().toLowerCase())
      setNewInboxInput('')
    }
  }

  return (
    <div className="flex flex-col h-full p-4">
      {/* Brand Header */}
      <div className="flex items-center gap-3 px-2 py-3 mb-2">
        <div className="w-10 h-10 rounded-xl bg-slate-950 dark:bg-white text-white dark:text-slate-950 flex items-center justify-center font-black text-xl shadow-md">
          <Mail className="w-5 h-5" />
        </div>
        <div>
          <h1 className="font-bold text-base leading-tight tracking-tight">Quick-Box</h1>
          <p className="text-xs text-slate-400">Developer Temporary Inbox</p>
        </div>
      </div>

      {/* Jump to Inbox Form */}
      <form onSubmit={handleCreateOrJump} className="my-3">
        <div className="relative flex items-center">
          <input
            type="text"
            value={newInboxInput}
            onChange={(e) => setNewInboxInput(e.target.value)}
            placeholder="Ketik nama inbox..."
            className="w-full pl-3 pr-10 py-2 text-xs rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 focus:outline-none focus:ring-2 focus:ring-slate-400 transition"
          />
          <button
            type="submit"
            className="absolute right-1.5 p-1 rounded-lg text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 cursor-pointer"
            title="Buka Inbox"
          >
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>
      </form>

      {/* Current Inbox Status Box */}
      <div className="p-3 my-2 rounded-xl bg-slate-50 dark:bg-slate-950/50 border border-slate-200 dark:border-slate-800/80">
        <div className="flex items-center justify-between mb-1">
          <span className="text-xs font-semibold text-slate-500 uppercase tracking-wider">Inbox Aktif</span>
          {status?.is_locked ? (
            <span className="inline-flex items-center gap-1 text-[11px] font-semibold text-rose-500">
              <Lock className="w-3 h-3" /> Terkunci
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-600 dark:text-emerald-400">
              <Unlock className="w-3 h-3" /> Publik
            </span>
          )}
        </div>
        <p className="font-bold text-sm text-slate-900 dark:text-slate-100 truncate mb-2.5">
          {currentInbox}
        </p>

        {status?.is_locked ? (
          <Button
            size="sm"
            variant="outline"
            className="w-full text-xs"
            onClick={onRemoveLock}
          >
            <Unlock className="w-3.5 h-3.5" /> Buka Kunci Sandi
          </Button>
        ) : (
          <Button
            size="sm"
            variant="secondary"
            className="w-full text-xs"
            onClick={onOpenLockModal}
          >
            <Lock className="w-3.5 h-3.5" /> Kunci dengan Sandi
          </Button>
        )}
      </div>

      {/* Recent Inboxes List */}
      <div className="flex-1 overflow-y-auto my-3 pr-1">
        <div className="flex items-center justify-between px-2 mb-2">
          <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
            Riwayat Inbox
          </span>
          <Clock className="w-3.5 h-3.5 text-slate-400" />
        </div>
        <div className="space-y-1">
          {recentInboxes.map((name) => (
            <button
              key={name}
              onClick={() => onSelectInbox(name)}
              className={`w-full text-left px-3 py-2 rounded-xl text-xs font-medium transition flex items-center justify-between cursor-pointer ${
                name === currentInbox
                  ? 'bg-slate-900 text-white dark:bg-slate-100 dark:text-slate-950 font-semibold shadow-xs'
                  : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800'
              }`}
            >
              <span className="truncate">{name}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Footer Info & Dark Mode */}
      <div className="pt-3 border-t border-slate-100 dark:border-slate-800/80 flex items-center justify-between text-xs text-slate-400">
        <div className="flex items-center gap-1.5 text-[11px]">
          <Shield className="w-3.5 h-3.5 text-emerald-500" />
          <span>Retensi 30 Hari</span>
        </div>
        <button
          onClick={onToggleDarkMode}
          className="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-500 cursor-pointer transition"
          title="Ganti Tema"
        >
          {darkMode ? <Sun className="w-4 h-4 text-amber-400" /> : <Moon className="w-4 h-4" />}
        </button>
      </div>
    </div>
  )
}
