import { useState, useEffect, useCallback } from 'react'
import { AppLayout } from './components/layout/AppLayout'
import { Sidebar } from './components/inbox/Sidebar'
import { MessageList } from './components/inbox/MessageList'
import { MessageDetail } from './components/viewer/MessageDetail'
import { LockModal } from './components/inbox/LockModal'
import { useInbox } from './hooks/useInbox'
import { useRealtimeEvents } from './hooks/useRealtimeEvents'

/** Normalisasi nama inbox:
 * - Strip bagian @domain jika user mengetik email lengkap (mis: sutrisno@gmail.com → sutrisno)
 * - Lowercase dan trim spasi
 */
function normalizeInboxName(raw: string): string {
  const trimmed = raw.trim().toLowerCase()
  const atIndex = trimmed.indexOf('@')
  return atIndex > 0 ? trimmed.slice(0, atIndex) : trimmed
}

export function App() {
  const [currentInbox, setCurrentInbox] = useState(() => {
    return localStorage.getItem('qb_last_inbox') || 'dev-test'
  })

  const [selectedMessageId, setSelectedMessageId] = useState<string | null>(null)
  const [recentInboxes, setRecentInboxes] = useState<string[]>(() => {
    try {
      return JSON.parse(localStorage.getItem('qb_recent_inboxes') || '["dev-test"]')
    } catch {
      return ['dev-test']
    }
  })

  const [darkMode, setDarkMode] = useState(() => {
    return localStorage.getItem('qb_dark_mode') === 'true'
  })

  const [lockModal, setLockModal] = useState<{ isOpen: boolean; mode: 'lock' | 'unlock' }>({
    isOpen: false,
    mode: 'lock',
  })

  // Dark mode effect
  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
    localStorage.setItem('qb_dark_mode', darkMode.toString())
  }, [darkMode])

  // Custom hook for active inbox
  const {
    status,
    messages,
    isLoading,
    isAuthRequired,
    refresh,
    lock,
    unlock,
    removeLock,
    markRead,
  } = useInbox(currentInbox)

  // Switch inbox handler
  const handleSelectInbox = useCallback((name: string) => {
    const clean = normalizeInboxName(name)
    if (!clean) return
    setCurrentInbox(clean)
    setSelectedMessageId(null)
    localStorage.setItem('qb_last_inbox', clean)

    setRecentInboxes((prev) => {
      const next = [clean, ...prev.filter((i) => i !== clean)].slice(0, 10)
      localStorage.setItem('qb_recent_inboxes', JSON.stringify(next))
      return next
    })
  }, [])

  // Realtime SSE event listener
  useRealtimeEvents(
    currentInbox,
    useCallback(
      (_event) => {
        // Refresh message list on new email arrival
        refresh()
      },
      [refresh]
    )
  )

  // Selected message detail
  const selectedMessage = messages.find((m) => m.id === selectedMessageId) || null

  const handleDeleteMessage = async (id: string) => {
    try {
      await fetch(`/api/inboxes/${encodeURIComponent(currentInbox)}/messages/${id}`, {
        method: 'DELETE',
      })
      setSelectedMessageId(null)
      refresh()
    } catch (e) {
      console.error('Failed to delete message', e)
    }
  }

  return (
    <>
      <AppLayout
        sidebar={
          <Sidebar
            currentInbox={currentInbox}
            status={status}
            recentInboxes={recentInboxes}
            onSelectInbox={handleSelectInbox}
            onOpenLockModal={() => setLockModal({ isOpen: true, mode: 'lock' })}
            onRemoveLock={removeLock}
            darkMode={darkMode}
            onToggleDarkMode={() => setDarkMode((prev) => !prev)}
          />
        }
        list={
          <MessageList
            currentInbox={currentInbox}
            messages={messages}
            selectedMessageId={selectedMessageId}
            onSelectMessage={(id) => {
                setSelectedMessageId(id)
                markRead(id)
              }}
            onRefresh={refresh}
            isLoading={isLoading}
            isAuthRequired={isAuthRequired}
            onOpenUnlockModal={() => setLockModal({ isOpen: true, mode: 'unlock' })}
          />
        }
        detail={
          <MessageDetail
            message={selectedMessage}
            onBack={() => setSelectedMessageId(null)}
            onDelete={handleDeleteMessage}
            inboxName={currentInbox}
          />
        }
        isReadingMobile={!!selectedMessageId}
      />

      <LockModal
        isOpen={lockModal.isOpen}
        onClose={() => setLockModal((prev) => ({ ...prev, isOpen: false }))}
        mode={lockModal.mode}
        inboxName={currentInbox}
        onSubmit={async (pwd) => {
          if (lockModal.mode === 'lock') {
            await lock(pwd)
          } else {
            await unlock(pwd)
          }
        }}
      />
    </>
  )
}

export default App
