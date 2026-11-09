import React, { useState, useMemo } from 'react'
import {
  Search,
  RefreshCw,
  Copy,
  Check,
  Star,
  Lock,
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
  isLoading: boolean
  isAuthRequired: boolean
  onOpenUnlockModal: () => void
}

export const MessageList: React.FC<MessageListProps> = ({
  currentInbox,
  messages,
  selectedMessageId,
  onSelectMessage,
  onRefresh,
  isLoading,
  isAuthRequired,
  onOpenUnlockModal,
}) => {
  const [filter, setFilter] = useState<'all' | 'unread' | 'favorites'>('all')
  const [searchQuery, setSearchQuery] = useState('')
  const [copied, setCopied] = useState(false)

  const emailAddress = `${currentInbox}@quickbox.dev`

  const handleCopy = () => {
    navigator.clipboard.writeText(emailAddress)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const filteredMessages = useMemo(() => {
    return messages.filter((m) => {
      if (filter === 'unread' && m.is_read) return false
      if (filter === 'favorites' && !m.is_favorite) return false
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase()
        const matchSender = m.sender.toLowerCase().includes(q)
        const matchSubj = m.subject.toLowerCase().includes(q)
        const matchSnippet = m.snippet.toLowerCase().includes(q)
        return matchSender || matchSubj || matchSnippet
      }
      return true
    })
  }, [messages, filter, searchQuery])

  // Get avatar initials & color
  const getAvatarInfo = (sender: string) => {
    const clean = sender.replace(/<.*>/, '').trim()
    const initial = clean.charAt(0).toUpperCase() || '?'
    const colors = [
      'bg-blue-500',
      'bg-indigo-500',
      'bg-violet-500',
      'bg-emerald-500',
      'bg-amber-500',
      'bg-rose-500',
    ]
    const code = clean.charCodeAt(0) || 0
    return { initial, color: colors[code % colors.length] }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Top Header: Current Email address & quick actions */}
      <div className="p-4 border-b border-slate-200 dark:border-slate-800/80 bg-white/70 dark:bg-slate-900/70 backdrop-blur-xs">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-1.5 min-w-0">
            <span className="font-bold text-sm text-slate-900 dark:text-slate-100 truncate">
              {emailAddress}
            </span>
            <button
              onClick={handleCopy}
              className="p-1 rounded text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 cursor-pointer transition"
              title="Salin Alamat Email"
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
            </button>
          </div>
          <Button
            size="sm"
            variant="ghost"
            onClick={onRefresh}
            className="p-1.5 h-8 w-8"
            title="Muat Ulang"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
        </div>

        {/* Search Bar */}
        <div className="relative">
          <Search className="w-4 h-4 absolute left-3 top-2.5 text-slate-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Cari email..."
            className="w-full pl-9 pr-3 py-1.5 text-xs rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 focus:outline-none focus:ring-1 focus:ring-slate-400 transition"
          />
        </div>

        {/* Filter Tabs */}
        <div className="flex items-center gap-1 mt-3">
          <button
            onClick={() => setFilter('all')}
            className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition ${
              filter === 'all'
                ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
                : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
            }`}
          >
            Semua ({messages.length})
          </button>
          <button
            onClick={() => setFilter('unread')}
            className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition ${
              filter === 'unread'
                ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
                : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
            }`}
          >
            Belum Dibaca
          </button>
          <button
            onClick={() => setFilter('favorites')}
            className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition ${
              filter === 'favorites'
                ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
                : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
            }`}
          >
            Favorit
          </button>
        </div>
      </div>

      {/* Messages List Area */}
      <div className="flex-1 overflow-y-auto divide-y divide-slate-100 dark:divide-slate-800/60">
        {isAuthRequired ? (
          <div className="p-8 text-center flex flex-col items-center justify-center h-full">
            <div className="w-12 h-12 rounded-full bg-rose-100 dark:bg-rose-950/40 text-rose-500 flex items-center justify-center mb-3">
              <Lock className="w-6 h-6" />
            </div>
            <h4 className="font-bold text-sm mb-1">Inbox Ini Terkunci</h4>
            <p className="text-xs text-slate-500 dark:text-slate-400 mb-4 max-w-xs">
              Masukkan kata sandi untuk melihat email yang masuk ke inbox ini.
            </p>
            <Button size="sm" onClick={onOpenUnlockModal}>
              Buka dengan Kata Sandi
            </Button>
          </div>
        ) : filteredMessages.length === 0 ? (
          <div className="p-8 text-center flex flex-col items-center justify-center h-full text-slate-400">
            <EmptyIcon className="w-10 h-10 mb-2 stroke-1" />
            <p className="text-sm font-medium">Belum ada email masuk</p>
            <p className="text-xs text-slate-400 mt-1 max-w-xs">
              Kirim email ke alamat ini dari aplikasi Anda. Email akan muncul seketika secara otomatis.
            </p>
          </div>
        ) : (
          filteredMessages.map((msg) => {
            const { initial, color } = getAvatarInfo(msg.sender)
            const isSelected = selectedMessageId === msg.id

            return (
              <div
                key={msg.id}
                onClick={() => onSelectMessage(msg.id)}
                className={`p-3.5 transition cursor-pointer flex gap-3 relative ${
                  isSelected
                    ? 'bg-white dark:bg-slate-800/80 shadow-xs'
                    : 'hover:bg-white/60 dark:hover:bg-slate-800/40'
                } ${!msg.is_read ? 'font-medium' : ''}`}
              >
                {/* Unread Blue Dot */}
                {!msg.is_read && (
                  <span className="w-2 h-2 rounded-full bg-blue-500 absolute top-4 left-2" />
                )}

                {/* Avatar */}
                <div
                  className={`w-9 h-9 rounded-full ${color} text-white flex items-center justify-center font-bold text-xs shrink-0 select-none shadow-xs`}
                >
                  {initial}
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
                    <span className="text-[11px] text-slate-400 shrink-0 ml-1">
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

                  <p className="text-xs text-slate-500 dark:text-slate-400 line-clamp-2 leading-relaxed">
                    {msg.snippet || '...'}
                  </p>

                  <div className="flex items-center gap-2 mt-1.5 text-[11px] text-slate-400">
                    {msg.detected_otp && (
                      <span className="font-mono bg-blue-50 dark:bg-blue-950/40 text-blue-600 dark:text-blue-400 px-1.5 py-0.5 rounded text-[10px] font-bold">
                        OTP {msg.detected_otp}
                      </span>
                    )}
                    {msg.is_favorite && (
                      <Star className="w-3 h-3 fill-amber-400 text-amber-400" />
                    )}
                  </div>
                </div>
              </div>
            )
          })
        )}
      </div>
    </div>
  )
}
