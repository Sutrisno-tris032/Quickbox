import React, { useState } from 'react'
import {
  Trash2,
  ArrowLeft,
  Calendar,
  User,
  Mail,
  FileCode,
  FileText,
  Layers,
} from 'lucide-react'
import { Button } from '../ui/Button'
import { OtpBanner } from './OtpBanner'
import { SandboxedHtmlFrame } from './SandboxedHtmlFrame'
import { AttachmentList } from './AttachmentList'
import type { Message } from '../../types'

export interface MessageDetailProps {
  message: Message | null
  onBack: () => void
  onDelete: (id: string) => void
  inboxName: string
}

export const MessageDetail: React.FC<MessageDetailProps> = ({
  message,
  onBack,
  onDelete,
  inboxName,
}) => {
  const [activeTab, setActiveTab] = useState<'html' | 'text' | 'headers'>('html')

  if (!message) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8 text-center text-slate-400">
        <Mail className="w-12 h-12 mb-3 stroke-1 text-slate-300 dark:text-slate-700" />
        <h3 className="font-semibold text-sm text-slate-600 dark:text-slate-400">
          Pilih email untuk dibaca
        </h3>
        <p className="text-xs text-slate-400 mt-1 max-w-xs">
          Klik salah satu pesan di daftar sebelah kiri untuk melihat konten lengkap, kode OTP, dan lampirannya.
        </p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Detail Header & Action Toolbar */}
      <div className="p-4 border-b border-slate-200 dark:border-slate-800/80 bg-white/70 dark:bg-slate-900/70 backdrop-blur-xs flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="ghost"
            onClick={onBack}
            className="md:hidden p-1.5 h-8 w-8"
            title="Kembali ke Daftar"
          >
            <ArrowLeft className="w-4 h-4" />
          </Button>
          <div className="flex items-center gap-1.5 text-xs text-slate-500">
            <Calendar className="w-3.5 h-3.5" />
            <span>
              {new Date(message.received_at).toLocaleDateString([], {
                day: 'numeric',
                month: 'short',
                year: 'numeric',
              })}{' '}
              {new Date(message.received_at).toLocaleTimeString([], {
                hour: '2-digit',
                minute: '2-digit',
              })}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-1.5">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => onDelete(message.id)}
            className="text-rose-500 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/30 h-8"
            title="Hapus Pesan"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span className="hidden sm:inline text-xs">Hapus</span>
          </Button>
        </div>
      </div>

      {/* Message Content Body */}
      <div className="flex-1 overflow-y-auto p-6 max-w-4xl w-full mx-auto">
        {/* Subject */}
        <h2 className="text-xl font-bold text-slate-900 dark:text-slate-100 mb-3 tracking-tight">
          {message.subject || '(Tanpa Subjek)'}
        </h2>

        {/* Sender & Recipient Metadata */}
        <div className="flex items-center gap-3 p-3 mb-5 rounded-xl bg-slate-50 dark:bg-slate-900/50 border border-slate-200/80 dark:border-slate-800 text-xs">
          <div className="w-9 h-9 rounded-full bg-slate-200 dark:bg-slate-800 flex items-center justify-center font-bold text-slate-600 dark:text-slate-300 shrink-0">
            <User className="w-4 h-4" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-slate-900 dark:text-slate-100 truncate">
                {message.sender}
              </span>
            </div>
            <div className="text-slate-400 truncate">Kepada: {message.recipient}</div>
          </div>
        </div>

        {/* OTP High-Priority Banner if detected */}
        {message.detected_otp && <OtpBanner otpCode={message.detected_otp} />}

        {/* Viewer Tabs (HTML / Text / Headers) */}
        <div className="flex items-center gap-2 mb-3 border-b border-slate-100 dark:border-slate-800 pb-2">
          {message.body_html && (
            <button
              onClick={() => setActiveTab('html')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition ${
                activeTab === 'html'
                  ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
                  : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
              }`}
            >
              <FileCode className="w-3.5 h-3.5" /> HTML Preview
            </button>
          )}

          <button
            onClick={() => setActiveTab('text')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition ${
              activeTab === 'text' || !message.body_html
                ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
                : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
            }`}
          >
            <FileText className="w-3.5 h-3.5" /> Plain Text
          </button>

          <button
            onClick={() => setActiveTab('headers')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition ${
              activeTab === 'headers'
                ? 'bg-slate-900 text-white dark:bg-white dark:text-slate-900'
                : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
            }`}
          >
            <Layers className="w-3.5 h-3.5" /> Raw Headers
          </button>
        </div>

        {/* Active Tab Panel */}
        <div className="min-h-[300px]">
          {activeTab === 'html' && message.body_html ? (
            <SandboxedHtmlFrame html={message.body_html} />
          ) : activeTab === 'text' || !message.body_html ? (
            <div className="p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900 text-xs font-mono whitespace-pre-wrap leading-relaxed text-slate-800 dark:text-slate-200">
              {message.body_text || '(Pesan kosong)'}
            </div>
          ) : (
            <div className="p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-900 text-emerald-400 text-xs font-mono whitespace-pre-wrap leading-relaxed overflow-x-auto">
              {message.raw_headers || '(Tidak ada header)'}
            </div>
          )}
        </div>

        {/* Attachments Section */}
        <AttachmentList inboxName={inboxName} messageId={message.id} />
      </div>
    </div>
  )
}

