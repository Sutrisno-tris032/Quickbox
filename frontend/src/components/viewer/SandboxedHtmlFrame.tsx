import React, { useState, useMemo } from 'react'
import { Image, ImageOff } from 'lucide-react'
import { Button } from '../ui/Button'

export interface SandboxedHtmlFrameProps {
  html: string
  inboxName?: string
}

export const SandboxedHtmlFrame: React.FC<SandboxedHtmlFrameProps> = ({ html, inboxName }) => {
  const [loadRemoteImages, setLoadRemoteImages] = useState(false)

  // Block external images by default unless user opts in
  const processedHtml = useMemo(() => {
    let sanitized = html

    if (!loadRemoteImages) {
      // Replace src="http..." on img tags with placeholder
      sanitized = sanitized.replace(
        /<img([^>]+)src=["'](https?:\/\/[^"']+)["']([^>]*)>/gi,
        '<img$1data-blocked-src="$2" src="data:image/svg+xml;utf8,<svg xmlns=\'http://www.w3.org/2000/svg\' width=\'100\' height=\'30\'><rect width=\'100%\' height=\'100%\' fill=\'%23eee\'/><text x=\'50%\' y=\'50%\' dominant-baseline=\'middle\' text-anchor=\'middle\' font-size=\'10\' fill=\'%23888\'>[Gambar Diblokir]</text></svg>"$3>'
      )
    }

    // Inline images (from cid: attachments) point at our own attachment endpoint,
    // not a third-party host, so they're always shown. The iframe uses srcDoc inside a
    // sandbox without allow-same-origin, so relative URLs never resolve to anything
    // fetchable — they must be rewritten to absolute URLs. If the inbox is locked, the
    // session token is appended too, same as attachment downloads.
    const token = inboxName ? sessionStorage.getItem(`qb_token_${inboxName.toLowerCase()}`) : null
    sanitized = sanitized.replace(
      /(src=["'])(\/api\/inboxes\/[^"']+\/attachments\/[^"'?]+)(["'])/gi,
      (_match, prefix, path, suffix) => {
        const absoluteUrl = `${window.location.origin}${path}`
        const withToken = token ? `${absoluteUrl}?token=${encodeURIComponent(token)}` : absoluteUrl
        return `${prefix}${withToken}${suffix}`
      }
    )

    // Add basic clean styling to HTML email inside iframe
    const styleHeader = `
      <style>
        body {
          font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
          font-size: 14px;
          line-height: 1.6;
          color: #1e293b;
          margin: 16px;
          word-break: break-word;
        }
        a { color: #2563eb; text-decoration: underline; }
        img { max-width: 100%; height: auto; }
      </style>
    `

    return `<!DOCTYPE html><html><head>${styleHeader}</head><body>${sanitized}</body></html>`
  }, [html, loadRemoteImages, inboxName])

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between pb-2 mb-2 border-b border-slate-100 dark:border-slate-800">
        <span className="text-xs text-slate-400">
          Render diisolasi via sandboxed iframe
        </span>
        <Button
          size="sm"
          variant="outline"
          className="text-xs h-7 gap-1.5"
          onClick={() => setLoadRemoteImages((prev) => !prev)}
        >
          {loadRemoteImages ? (
            <>
              <ImageOff className="w-3.5 h-3.5 text-amber-500" /> Blokir Gambar Luar
            </>
          ) : (
            <>
              <Image className="w-3.5 h-3.5" /> Muat Gambar Luar
            </>
          )}
        </Button>
      </div>

      <iframe
        title="Email HTML Preview"
        sandbox="allow-popups"
        srcDoc={processedHtml}
        className="w-full flex-1 min-h-[400px] rounded-xl border border-slate-200 dark:border-slate-800 bg-white"
      />
    </div>
  )
}

