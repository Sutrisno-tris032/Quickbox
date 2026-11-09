import React, { useState, useEffect } from 'react'
import { Download, FileText, Paperclip } from 'lucide-react'
import type { Attachment } from '../../types'

export interface AttachmentListProps {
  inboxName: string
  messageId: string
}

export const AttachmentList: React.FC<AttachmentListProps> = ({
  inboxName,
  messageId,
}) => {
  const [attachments, setAttachments] = useState<Attachment[]>([])

  useEffect(() => {
    const fetchAttachments = async () => {
      try {
        const token = sessionStorage.getItem(`qb_token_${inboxName.toLowerCase()}`) || ''
        const headers: Record<string, string> = {}
        if (token) headers['Authorization'] = `Bearer ${token}`

        const res = await fetch(
          `/api/inboxes/${encodeURIComponent(inboxName)}/messages/${messageId}/attachments`,
          { headers }
        )
        if (res.ok) {
          const data = await res.json()
          setAttachments(data)
        }
      } catch (e) {
        console.debug('Failed to fetch attachments', e)
      }
    }
    fetchAttachments()
  }, [inboxName, messageId])

  if (attachments.length === 0) return null

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  return (
    <div className="mt-4 pt-4 border-t border-slate-100 dark:border-slate-800">
      <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-500 uppercase tracking-wider mb-2.5">
        <Paperclip className="w-3.5 h-3.5" />
        <span>Lampiran Berkas ({attachments.length})</span>
      </div>
      <div className="flex flex-wrap gap-2">
        {attachments.map((att) => (
          <a
            key={att.id}
            href={`/api/inboxes/${encodeURIComponent(inboxName)}/attachments/${att.id}`}
            download={att.filename}
            className="flex items-center gap-2.5 px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800 transition text-xs font-medium text-slate-700 dark:text-slate-300 group"
          >
            <FileText className="w-4 h-4 text-slate-400 group-hover:text-blue-500 transition" />
            <div className="flex flex-col text-left">
              <span className="truncate max-w-[140px] font-semibold">{att.filename}</span>
              <span className="text-[10px] text-slate-400">{formatSize(att.size_bytes)}</span>
            </div>
            <Download className="w-3.5 h-3.5 text-slate-400 group-hover:text-slate-700 dark:group-hover:text-slate-200 ml-1" />
          </a>
        ))}
      </div>
    </div>
  )
}
