import React, { useState } from 'react'
import {
  ArrowLeft,
  Copy,
  Check,
  Mail,
} from 'lucide-react'
import { Button } from '../ui/Button'
import { SandboxedHtmlFrame } from './SandboxedHtmlFrame'
import { AttachmentList } from './AttachmentList'
import type { Message } from '../../types'

export interface MessageDetailProps {
  message: Message | null
  onBack: () => void
  inboxName: string
}

export const MessageDetail: React.FC<MessageDetailProps> = ({
  message,
  onBack,
  inboxName,
}) => {
  const [activeTab, setActiveTab] = useState<'html' | 'text' | 'headers'>('html')
  const [copiedEmail, setCopiedEmail] = useState(false)

  const DOMAIN = import.meta.env.VITE_DOMAIN || 'quickbox.trisatya.my.id'
  const emailAddress = `${inboxName}@${DOMAIN}`

  const handleCopyEmail = () => {
    navigator.clipboard.writeText(emailAddress)
    setCopiedEmail(true)
    setTimeout(() => setCopiedEmail(false), 2000)
  }

  // Get sender initials for reader badge
  const senderInitial = message?.sender ? message.sender.slice(0, 2).toUpperCase() : 'EM'

  return (
    <div className="flex flex-col h-full overflow-hidden bg-[#F8FAFC] dark:bg-[#0B0F19]">
      {/* 1. Top Section: Alamat Sementara Aktif Card (Clean Hero Banner) */}
      <div className="p-4 border-b border-slate-200/80 dark:border-slate-800 bg-white dark:bg-[#0F172A] shrink-0">
        <div className="flex items-center justify-between text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-2">
          <span className="flex items-center gap-1.5 text-blue-600 dark:text-blue-400">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
            Alamat Sementara Aktif
          </span>
          <span className="font-mono text-slate-400">Retensi: 30 Hari</span>
        </div>

        {/* Address Bar with Copy */}
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 mb-3">
          <div className="flex-1 flex items-center gap-2 px-3 py-2 rounded-xl border border-blue-200/80 dark:border-blue-900/60 bg-blue-50/30 dark:bg-blue-950/20 text-blue-950 dark:text-blue-100 font-mono text-sm font-bold min-w-0">
            <Mail className="w-4 h-4 text-blue-600 shrink-0" />
            <span className="truncate">{emailAddress}</span>
          </div>

          <Button
            size="sm"
            onClick={handleCopyEmail}
            className="bg-blue-600 hover:bg-blue-700 text-white font-semibold text-xs px-4 py-2 rounded-xl gap-1.5 shrink-0 shadow-xs cursor-pointer"
          >
            {copiedEmail ? (
              <>
                <Check className="w-4 h-4 text-emerald-300" />
                <span>Tersalin!</span>
              </>
            ) : (
              <>
                <Copy className="w-4 h-4" />
                <span>Salin Alamat</span>
              </>
            )}
          </Button>
        </div>
      </div>

      {/* 2. Message Detail Body Area */}
      <div className="flex-1 overflow-y-auto p-4 sm:p-6">
        {!message ? (
          <div className="flex flex-col items-center justify-center h-full p-8 text-center text-slate-400">
            <div className="w-16 h-16 rounded-2xl bg-white dark:bg-[#0F172A] border border-slate-200 dark:border-slate-800 flex items-center justify-center mb-3 shadow-xs">
              <Mail className="w-8 h-8 text-slate-300 dark:text-slate-600 stroke-1" />
            </div>
            <h3 className="font-bold text-sm text-slate-700 dark:text-slate-300">
              Pilih email untuk dibaca
            </h3>
            <p className="text-xs text-slate-400 mt-1 max-w-sm leading-relaxed">
              Semua email masuk ke <span className="font-mono text-blue-600">{emailAddress}</span> akan ditampilkan dengan preview HTML bersih dan deteksi OTP instan.
            </p>
          </div>
        ) : (
          <div className="max-w-4xl mx-auto space-y-4">
            {/* Header: Sender badge, Subject, Action buttons */}
            <div className="p-4 rounded-2xl bg-white dark:bg-[#0F172A] border border-slate-200/80 dark:border-slate-800 shadow-xs">
              <div className="flex items-start gap-3 mb-3">
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={onBack}
                  className="md:hidden p-1.5 h-8 w-8 -ml-1"
                >
                  <ArrowLeft className="w-4 h-4" />
                </Button>
                <div className="w-10 h-10 rounded-xl bg-slate-900 dark:bg-white text-white dark:text-slate-900 flex items-center justify-center font-bold text-sm shrink-0 shadow-xs">
                  {senderInitial}
                </div>
                <div>
                  <h2 className="font-bold text-base text-slate-900 dark:text-white leading-snug">
                    {message.subject || '(Tanpa Subjek)'}
                  </h2>
                  <div className="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400 mt-0.5">
                    <span className="font-medium text-slate-700 dark:text-slate-300">
                      {message.sender}
                    </span>
                    <span>•</span>
                    <span>
                      {new Date(message.received_at).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit',
                      })} WIB
                    </span>
                  </div>
                </div>
              </div>
            </div>

            {/* OTP Clean Card Banner (Match Reference Image) */}
            {message.detected_otp && (
              <div className="p-6 rounded-2xl bg-white dark:bg-[#0F172A] border border-blue-100 dark:border-blue-900/50 shadow-xs text-center flex flex-col items-center">
                <span className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase mb-2">
                  Kode Verifikasi Sementara (Valid 10 Menit)
                </span>
                <div className="flex items-center gap-3 mb-2">
                  <span className="font-mono text-3xl sm:text-4xl font-extrabold tracking-widest text-slate-900 dark:text-white">
                    {message.detected_otp}
                  </span>
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(message.detected_otp!)
                      alert('Kode OTP disalin!')
                    }}
                    className="p-2 rounded-xl border border-slate-200 dark:border-slate-800 hover:bg-slate-100 dark:hover:bg-slate-800 transition cursor-pointer"
                    title="Salin OTP"
                  >
                    <Copy className="w-4 h-4 text-blue-600" />
                  </button>
                </div>
                <p className="text-[11px] text-emerald-600 dark:text-emerald-400 flex items-center gap-1 font-medium">
                  ⚡ Terdeteksi otomatis • Klik tombol salin untuk verifikasi cepat
                </p>
              </div>
            )}

            {/* Viewer Mode Switcher & Content */}
            <div className="p-4 rounded-2xl bg-white dark:bg-[#0F172A] border border-slate-200/80 dark:border-slate-800 shadow-xs">
              <div className="flex items-center gap-2 mb-4 border-b border-slate-100 dark:border-slate-800 pb-3">
                <button
                  onClick={() => setActiveTab('html')}
                  className={`px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition ${
                    activeTab === 'html'
                      ? 'bg-blue-600 text-white'
                      : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
                  }`}
                >
                  Rendered HTML
                </button>
                <button
                  onClick={() => setActiveTab('text')}
                  className={`px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition ${
                    activeTab === 'text'
                      ? 'bg-blue-600 text-white'
                      : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
                  }`}
                >
                  Raw Source / Text
                </button>
              </div>

              {/* Render Tab Contents */}
              {activeTab === 'html' && message.body_html ? (
                <SandboxedHtmlFrame html={message.body_html} inboxName={inboxName} />
              ) : activeTab === 'text' || !message.body_html ? (
                <pre className="p-4 rounded-xl bg-slate-50 dark:bg-[#0B0F19] text-xs font-mono text-slate-800 dark:text-slate-200 whitespace-pre-wrap overflow-x-auto leading-relaxed border border-slate-200/60 dark:border-slate-800">
                  {message.body_text || message.snippet || '(Tidak ada konten teks)'}
                </pre>
              ) : null}

              {/* Attachments Section */}
              <AttachmentList inboxName={inboxName} messageId={message.id} />
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
