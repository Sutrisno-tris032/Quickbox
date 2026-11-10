import React, { useState, useMemo } from 'react'
import {
  RefreshCw,
  Star,
  Lock,
  Trash2,
  Inbox as EmptyIcon,
} from 'lucide-react'
import { Button } from '../ui/Button'
import type { Message } from '../../types'

export interface MessageListProps {
  currentInbox: string
  messages: Message[]
  selectedMessageId: string | null
  onSelectMessage: (id: string) => void
  onRefresh: () => void
  onClearAllMessages?: () => void
  isLoading: boolean
  isAuthRequired: boolean
  onOpenUnlockModal: () => void
}

export const MessageList: React.FC<MessageListProps> = ({
  messages,
  selectedMessageId,
  onSelectMessage,
  onRefresh,
  onClearAllMessages,
  isLoading,
  isAuthRequired,
  onOpenUnlockModal,
}) => {
  const [filter, setFilter] = useState<'all' | 'unread' | 'favorites'>('all')

  const unreadCount = useMemo(() => messages.filter((m) => !m.is_read).length, [messages])

  const filteredMessages = useMemo(() => {
    return messages.filter((m) => {
      if (filter === 'unread' && m.is_read) return false
      if (filter === 'favorites' && !m.is_favorite) return false
      return true
    })
  }, [messages, filter])

  // Get avatar badge based on sender name
  const getAvatarBadge = (sender: string) => {
    const clean = sender.replace(/<.*>/, '').trim()
    const firstTwo = clean.slice(0, 2).toUpperCase() || 'EM'
    const colors = [
      'bg-slate-900 text-white dark:bg-slate-100 dark:text-slate-950',
      'bg-blue-600 text-white',
      'bg-indigo-600 text-white',
      'bg-violet-600 text-white',
      'bg-emerald-600 text-white',
    ]
    const code = clean.charCodeAt(0) || 0
    return { label: firstTwo, colorClass: colors[code % colors.length] }
  }

  return (
    <div className="flex flex-col h-full bg-white dark:bg-[#0F172A]">
      {/* Top Header: Title & Action Icons */}
      <div className="p-3.5 border-b border-slate-200/80 dark:border-slate-800">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <h2 className="font-bold text-sm text-slate-900 dark:text-white">
              Pesan Masuk
            </h2>
            <span className="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-400">
              {messages.length}
            </span>
          </div>

          <div className="flex items-center gap-1">
            <Button
              size="sm"
              variant="ghost"
              onClick={onRefresh}
              className="p-1 h-7 w-7 text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
              title="Perbarui"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
            </Button>
            {messages.length > 0 && onClearAllMessages && (
              <Button
                size="sm"
                variant="ghost"
                onClick={onClearAllMessages}
                className="p-1 h-7 w-7 text-slate-400 hover:text-rose-500"
                title="Hapus Semua Pesan"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </Button>
            )}
          </div>
        </div>

        {/* Sub-label sync info */}
        <div className="flex items-center justify-between text-[10px] text-slate-400 mb-2.5">
          <span className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-blue-500" />
            Auto-refresh aktif
          </span>
          <span>Protokol SSE/Web</span>
        </div>

        {/* 3 Clean Filter Tabs */}
        <div className="grid grid-cols-3 gap-1 p-0.5 rounded-xl bg-slate-100 dark:bg-slate-800/60 text-xs font-semibold">
          <button
            onClick={() => setFilter('all')}
            className={`py-1.5 text-center rounded-lg transition cursor-pointer ${
              filter === 'all'
                ? 'bg-blue-600 text-white shadow-xs'
                : 'text-slate-500 hover:text-slate-900 dark:hover:text-white'
            }`}
          >
            Semua ({messages.length})
          </button>
          <button
            onClick={() => setFilter('unread')}
            className={`py-1.5 text-center rounded-lg transition cursor-pointer ${
              filter === 'unread'
                ? 'bg-blue-600 text-white shadow-xs'
                : 'text-slate-500 hover:text-slate-900 dark:hover:text-white'
            }`}
          >
            Belum Dibaca ({unreadCount})
          </button>
          <button
            onClick={() => setFilter('favorites')}
            className={`py-1.5 text-center rounded-lg transition cursor-pointer ${
              filter === 'favorites'
                ? 'bg-blue-600 text-white shadow-xs'
                : 'text-slate-500 hover:text-slate-900 dark:hover:text-white'
            }`}
          >
            Berbintang
          </button>
        </div>
      </div>

      {/* Messages List Area */}
      <div className="flex-1 overflow-y-auto divide-y divide-slate-100 dark:divide-slate-800/50">
        {isAuthRequired ? (
          <div className="p-8 text-center flex flex-col items-center justify-center h-full">
            <div className="w-12 h-12 rounded-2xl bg-rose-50 dark:bg-rose-950/40 text-rose-500 flex items-center justify-center mb-3">
              <Lock className="w-6 h-6" />
            </div>
            <h4 className="font-bold text-sm mb-1">Inbox Ini Terkunci</h4>
            <p className="text-xs text-slate-500 dark:text-slate-400 mb-4 max-w-xs">
              Masukkan kata sandi untuk melihat email yang masuk ke inbox ini.
            </p>
            <Button size="sm" onClick={onOpenUnlockModal} className="bg-blue-600 hover:bg-blue-700 text-white">
              Buka dengan Kata Sandi
            </Button>
          </div>
        ) : filteredMessages.length === 0 ? (
          <div className="p-8 text-center flex flex-col items-center justify-center h-full text-slate-400">
            <EmptyIcon className="w-10 h-10 mb-2 stroke-1 text-slate-300 dark:text-slate-700" />
            <p className="text-xs font-semibold text-slate-600 dark:text-slate-400">Belum ada email masuk</p>
            <p className="text-[11px] text-slate-400 mt-1 max-w-xs leading-relaxed">
              Kirim email ke alamat aktif di atas dari aplikasi Anda.
            </p>
          </div>
        ) : (
          filteredMessages.map((msg) => {
            const isSelected = selectedMessageId === msg.id
            const { label, colorClass } = getAvatarBadge(msg.sender)

            return (
              <div
                key={msg.id}
                onClick={() => onSelectMessage(msg.id)}
                className={`p-3.5 transition cursor-pointer flex gap-3 relative ${
                  isSelected
                    ? 'bg-blue-50/60 dark:bg-blue-950/30 border-l-3 border-blue-600'
                    : 'hover:bg-slate-50/80 dark:hover:bg-slate-800/40'
                }`}
              >
                {/* Square Avatar Badge */}
                <div
                  className={`w-9 h-9 rounded-lg ${colorClass} flex items-center justify-center font-bold text-xs shrink-0 select-none shadow-2xs`}
                >
                  {label}
                </div>

                {/* Details */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between mb-0.5">
                    <span
                      className={`text-xs truncate ${
                        !msg.is_read
                          ? 'font-bold text-slate-900 dark:text-slate-100'
                          : 'text-slate-600 dark:text-slate-400'
                      }`}
                    >
                      {msg.sender.replace(/<.*>/, '').trim() || msg.sender}
                    </span>
                    <span className="text-[10px] text-slate-400 shrink-0 ml-1">
                      {new Date(msg.received_at).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </span>
                  </div>

                  <h5
                    className={`text-xs mb-1 truncate ${
                      !msg.is_read
                        ? 'font-semibold text-slate-800 dark:text-slate-200'
                        : 'text-slate-600 dark:text-slate-400'
                    }`}
                  >
                    {msg.subject || '(Tanpa Subjek)'}
                  </h5>

                  <p className="text-[11px] text-slate-500 dark:text-slate-400 line-clamp-2 leading-relaxed">
                    {msg.snippet || '...'}
                  </p>

                  <div className="flex items-center gap-1.5 mt-2">
                    {msg.detected_otp && (
                      <span className="font-mono bg-blue-50 dark:bg-blue-950/60 text-blue-600 dark:text-blue-400 px-2 py-0.5 rounded text-[10px] font-bold">
                        OTP {msg.detected_otp}
                      </span>
                    )}
                    <span className="text-[10px] px-1.5 py-0.2 rounded border border-slate-200 dark:border-slate-800 text-slate-500">
                      Verification
                    </span>
                    {msg.is_favorite && <Star className="w-3 h-3 fill-amber-400 text-amber-400" />}
                  </div>
                </div>

                {/* Unread Indicator */}
                {!msg.is_read && (
                  <span className="w-1.5 h-1.5 rounded-full bg-blue-600 absolute top-4 right-3" />
                )}
              </div>
            )
          })
        )}
      </div>

      {/* Footer Info */}
      <div className="p-2 border-t border-slate-100 dark:border-slate-800/80 text-center text-[10px] text-slate-400 bg-slate-50/50 dark:bg-slate-900/30">
        Listening for inbound transmissions...
      </div>
    </div>
  )
}
