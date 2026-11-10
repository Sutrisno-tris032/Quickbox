import React, { useState } from 'react'
import {
  Search,
  Lock,
  Unlock,
  Trash2,
  X,
  ShieldCheck,
  HardDrive,
  Sparkles,
} from 'lucide-react'
import { Button } from '../ui/Button'
import type { InboxStatus } from '../../types'

export interface SidebarProps {
  currentInbox: string
  status: InboxStatus | null
  recentInboxes: string[]
  onSelectInbox: (name: string) => void
  onRemoveRecentInbox?: (name: string) => void
  onClearAllRecentInboxes?: () => void
  onOpenLockModal: () => void
  onRemoveLock: () => void
  onGenerateRandomInbox?: () => void
  darkMode: boolean
  onToggleDarkMode: () => void
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentInbox,
  status,
  recentInboxes,
  onSelectInbox,
  onRemoveRecentInbox,
  onClearAllRecentInboxes,
  onOpenLockModal,
  onRemoveLock,
  onGenerateRandomInbox,
}) => {
  const [searchInput, setSearchInput] = useState('')

  const filteredInboxes = recentInboxes.filter((name) =>
    name.toLowerCase().includes(searchInput.toLowerCase())
  )

  const handleCreateOrJump = (e: React.FormEvent) => {
    e.preventDefault()
    if (searchInput.trim()) {
      const raw = searchInput.trim().toLowerCase()
      const atIndex = raw.indexOf('@')
      const clean = atIndex > 0 ? raw.slice(0, atIndex) : raw
      if (clean) {
        onSelectInbox(clean)
        setSearchInput('')
      }
    }
  }

  return (
    <div className="flex flex-col h-full p-3.5 justify-between">
      {/* Top Header & New Generation Button */}
      <div>
        <div className="flex items-center justify-between mb-2 px-1">
          <div>
            <h2 className="font-bold text-sm text-slate-900 dark:text-white leading-tight">
              Riwayat Inbox
            </h2>
            <p className="text-[11px] text-slate-400">Generated Mailboxes</p>
          </div>
          <span className="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-500">
            {recentInboxes.length} Total
          </span>
        </div>

        {/* Generate / New Address Action */}
        <Button
          onClick={onGenerateRandomInbox}
          size="sm"
          className="w-full justify-center gap-2 bg-blue-600 hover:bg-blue-700 text-white font-semibold text-xs py-2 rounded-xl mb-3 shadow-xs cursor-pointer"
        >
          <Sparkles className="w-3.5 h-3.5" />
          <span>Generate New Address</span>
        </Button>

        {/* Search / Jump Input */}
        <form onSubmit={handleCreateOrJump} className="relative mb-3">
          <Search className="w-3.5 h-3.5 absolute left-3 top-2.5 text-slate-400" />
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            placeholder="Cari sesi alamat / label..."
            className="w-full pl-8 pr-3 py-1.5 text-xs rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-[#0B0F19] text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500 transition"
          />
        </form>

        {/* Active Inbox Card with Lock & Status */}
        <div className="p-3 rounded-xl bg-white dark:bg-[#0F172A] border border-blue-200 dark:border-blue-950/80 shadow-xs mb-3">
          <div className="flex items-center justify-between text-[11px] mb-1">
            <span className="inline-flex items-center gap-1.5 font-semibold text-blue-600 dark:text-blue-400">
              <span className="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse" />
              Aktif (Active)
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-400 font-medium">
              Sesi Utama
            </span>
          </div>

          <div className="font-bold text-xs text-slate-900 dark:text-slate-100 truncate mb-2">
            {currentInbox}
          </div>

          <div className="flex items-center justify-between pt-2 border-t border-slate-100 dark:border-slate-800/80">
            <span className="text-[10px] text-slate-400 flex items-center gap-1">
              {status?.is_locked ? (
                <span className="text-rose-500 flex items-center gap-1 font-medium">
                  <Lock className="w-3 h-3" /> Terkunci Sandi
                </span>
              ) : (
                <span className="text-emerald-500 flex items-center gap-1 font-medium">
                  <Unlock className="w-3 h-3" /> Publik (Dev)
                </span>
              )}
            </span>

            {status?.is_locked ? (
              <button
                onClick={onRemoveLock}
                className="text-[11px] font-semibold text-rose-600 hover:underline cursor-pointer"
              >
                Buka
              </button>
            ) : (
              <button
                onClick={onOpenLockModal}
                className="text-[11px] font-semibold text-blue-600 dark:text-blue-400 hover:underline cursor-pointer"
              >
                Kunci
              </button>
            )}
          </div>
        </div>

        {/* Inboxes List */}
        <div className="overflow-y-auto max-h-[calc(100vh-420px)] space-y-1.5 pr-1">
          {filteredInboxes.map((name) => {
            const isCurrent = name === currentInbox
            return (
              <div
                key={name}
                onClick={() => onSelectInbox(name)}
                className={`group p-2.5 rounded-xl border text-xs transition cursor-pointer flex flex-col gap-1 ${
                  isCurrent
                    ? 'border-blue-500/80 bg-blue-50/50 dark:bg-blue-950/20 shadow-xs'
                    : 'border-slate-200/60 dark:border-slate-800/60 bg-white dark:bg-[#0F172A] hover:border-slate-300 dark:hover:border-slate-700'
                }`}
              >
                <div className="flex items-center justify-between">
                  <span className="font-semibold truncate text-slate-900 dark:text-slate-200">
                    {name}
                  </span>
                  {onRemoveRecentInbox && (
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation()
                        onRemoveRecentInbox(name)
                      }}
                      className="opacity-0 group-hover:opacity-100 p-0.5 rounded text-slate-400 hover:text-rose-500 transition cursor-pointer"
                      title="Hapus sesi"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  )}
                </div>
                <div className="flex items-center justify-between text-[10px] text-slate-400">
                  <span>Tersimpan di browser</span>
                  {isCurrent && <span className="font-bold text-blue-600">Dipilih</span>}
                </div>
              </div>
            )
          })}
        </div>
      </div>

      {/* Bottom Capacity & Privacy Card */}
      <div className="space-y-3 pt-3 border-t border-slate-200/80 dark:border-slate-800/80">
        <div className="p-2.5 rounded-xl border border-slate-200/70 dark:border-slate-800 bg-white dark:bg-[#0F172A] text-[11px]">
          <div className="flex items-center justify-between mb-1 text-slate-500">
            <span className="flex items-center gap-1 font-semibold text-[10px] uppercase">
              <HardDrive className="w-3 h-3" /> Kapasitas Sesi Lokal
            </span>
            <span>{recentInboxes.length} / 10 Max</span>
          </div>
          <div className="flex items-center justify-between text-[10px] text-slate-400 mt-2">
            <span>Auto-purge 30 Hari</span>
            {onClearAllRecentInboxes && recentInboxes.length > 0 && (
              <button
                onClick={onClearAllRecentInboxes}
                className="text-rose-500 hover:underline cursor-pointer flex items-center gap-1 font-medium"
              >
                <Trash2 className="w-2.5 h-2.5" /> Hapus Semua
              </button>
            )}
          </div>
        </div>

        <div className="p-2.5 rounded-xl bg-slate-100/70 dark:bg-slate-900/40 text-[10px] text-slate-500 dark:text-slate-400 leading-relaxed">
          <div className="flex items-center gap-1 text-slate-700 dark:text-slate-300 font-semibold mb-0.5">
            <ShieldCheck className="w-3 h-3 text-emerald-500" />
            <span>Privasi Terjamin</span>
          </div>
          Pesan masuk dianalisis secara lokal tanpa tracking pihak ketiga. Kode OTP diekstrak otomatis.
        </div>
      </div>
    </div>
  )
}
