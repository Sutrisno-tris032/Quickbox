import React, { useState } from 'react'
import { Modal } from '../ui/Modal'
import { Button } from '../ui/Button'
import { Lock, AlertCircle } from 'lucide-react'

export interface LockModalProps {
  isOpen: boolean
  onClose: () => void
  mode: 'lock' | 'unlock'
  inboxName: string
  onSubmit: (password: string) => Promise<void>
}

export const LockModal: React.FC<LockModalProps> = ({
  isOpen,
  onClose,
  mode,
  inboxName,
  onSubmit,
}) => {
  const [password, setPassword] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!password.trim()) {
      setError('Kata sandi tidak boleh kosong')
      return
    }

    setIsLoading(true)
    setError(null)
    try {
      await onSubmit(password)
      setPassword('')
      onClose()
    } catch (err: any) {
      setError(err.message || 'Terjadi kesalahan')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={mode === 'lock' ? `Kunci Inbox "${inboxName}"` : `Buka Inbox "${inboxName}"`}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {mode === 'lock' && (
          <div className="p-3 rounded-xl bg-amber-50 dark:bg-amber-950/40 border border-amber-200 dark:border-amber-800/80 text-amber-800 dark:text-amber-300 text-xs flex gap-2">
            <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
            <p>
              Mengunci inbox akan membersihkan email publik sebelumnya untuk privasi, dan hanya mereka yang memiliki kata sandi ini yang dapat membaca email baru.
            </p>
          </div>
        )}

        <div>
          <label className="block text-xs font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
            Kata Sandi
          </label>
          <div className="relative">
            <Lock className="w-4 h-4 absolute left-3 top-3 text-slate-400" />
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Masukkan kata sandi..."
              className="w-full pl-9 pr-3 py-2 text-sm rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 focus:outline-none focus:ring-2 focus:ring-slate-400"
              autoFocus
            />
          </div>
        </div>

        {error && <p className="text-xs text-rose-500 font-medium">{error}</p>}

        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="outline" onClick={onClose} disabled={isLoading}>
            Batal
          </Button>
          <Button type="submit" disabled={isLoading}>
            {isLoading ? 'Memproses...' : mode === 'lock' ? 'Kunci Inbox' : 'Buka Kunci'}
          </Button>
        </div>
      </form>
    </Modal>
  )
}

