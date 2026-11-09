import React, { useState } from 'react'
import { KeyRound, Copy, Check } from 'lucide-react'
import { Button } from '../ui/Button'

export interface OtpBannerProps {
  otpCode: string
}

export const OtpBanner: React.FC<OtpBannerProps> = ({ otpCode }) => {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(otpCode)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="flex items-center justify-between p-3.5 mb-4 rounded-xl bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-950/40 dark:to-indigo-950/30 border border-blue-200 dark:border-blue-900/60 shadow-xs">
      <div className="flex items-center gap-3">
        <div className="w-9 h-9 rounded-xl bg-blue-600 text-white flex items-center justify-center shrink-0 shadow-xs">
          <KeyRound className="w-5 h-5" />
        </div>
        <div>
          <span className="text-xs font-semibold text-blue-700 dark:text-blue-300 uppercase tracking-wider block">
            Kode OTP Terdeteksi
          </span>
          <span className="font-mono text-xl font-black text-slate-900 dark:text-white tracking-widest">
            {otpCode}
          </span>
        </div>
      </div>
      <Button
        size="sm"
        onClick={handleCopy}
        className="bg-blue-600 hover:bg-blue-700 text-white border-0 shadow-xs"
      >
        {copied ? (
          <>
            <Check className="w-3.5 h-3.5 text-emerald-300" /> Tersalin!
          </>
        ) : (
          <>
            <Copy className="w-3.5 h-3.5" /> Salin Kode
          </>
        )}
      </Button>
    </div>
  )
}

