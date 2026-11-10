import { useState, useEffect, useCallback } from 'react'
import type { InboxStatus, Message } from '../types'

export function useInbox(inboxName: string) {
  const [status, setStatus] = useState<InboxStatus | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [isAuthRequired, setIsAuthRequired] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const getToken = useCallback(() => {
    return sessionStorage.getItem(`qb_token_${inboxName.toLowerCase()}`) || ''
  }, [inboxName])

  const setToken = useCallback((token: string) => {
    if (token) {
      sessionStorage.setItem(`qb_token_${inboxName.toLowerCase()}`, token)
    } else {
      sessionStorage.removeItem(`qb_token_${inboxName.toLowerCase()}`)
    }
  }, [inboxName])

  const fetchStatus = useCallback(async () => {
    try {
      const res = await fetch(`/api/inboxes/${encodeURIComponent(inboxName)}/status`)
      if (res.ok) {
        const data: InboxStatus = await res.json()
        setStatus(data)
        return data
      }
    } catch (e: any) {
      console.error('Failed to fetch inbox status', e)
    }
    return null
  }, [inboxName])

  const fetchMessages = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    const token = getToken()
    const headers: Record<string, string> = {}
    if (token) {
      headers['Authorization'] = `Bearer ${token}`
    }

    try {
      const res = await fetch(`/api/inboxes/${encodeURIComponent(inboxName)}/messages`, { headers })
      if (res.status === 401) {
        setIsAuthRequired(true)
        setMessages([])
      } else if (res.ok) {
        setIsAuthRequired(false)
        const data: Message[] = await res.json()
        setMessages(data)
      } else {
        const err = await res.json()
        setError(err.error || 'Failed to load messages')
      }
    } catch (e: any) {
      setError(e.message)
    } finally {
      setIsLoading(false)
    }
  }, [inboxName, getToken])

  const refresh = useCallback(async () => {
    await fetchStatus()
    await fetchMessages()
  }, [fetchStatus, fetchMessages])

  const lock = async (password: string) => {
    const res = await fetch(`/api/inboxes/${encodeURIComponent(inboxName)}/lock`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ password }),
    })
    if (!res.ok) {
      const data = await res.json()
      throw new Error(data.error || 'Failed to lock inbox')
    }
    // Automatically unlock to get token
    await unlock(password)
    await refresh()
  }

  const unlock = async (password: string) => {
    const res = await fetch(`/api/inboxes/${encodeURIComponent(inboxName)}/unlock`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ password }),
    })
    if (!res.ok) {
      const data = await res.json()
      throw new Error(data.error || 'Invalid password')
    }
    const data = await res.json()
    setToken(data.token)
    setIsAuthRequired(false)
    await refresh()
  }

  const removeLock = async () => {
    const token = getToken()
    const headers: Record<string, string> = {}
    if (token) headers['Authorization'] = `Bearer ${token}`

    const res = await fetch(`/api/inboxes/${encodeURIComponent(inboxName)}/lock`, {
      method: 'DELETE',
      headers,
    })
    if (!res.ok) {
      const data = await res.json()
      throw new Error(data.error || 'Failed to remove lock')
    }
    setToken('')
    await refresh()
  }

  const clearMessages = async () => {
    const token = getToken()
    const headers: Record<string, string> = {}
    if (token) headers['Authorization'] = `Bearer ${token}`

    const res = await fetch(`/api/inboxes/${encodeURIComponent(inboxName)}/messages`, {
      method: 'DELETE',
      headers,
    })
    if (!res.ok) {
      const data = await res.json()
      throw new Error(data.error || 'Failed to clear inbox messages')
    }
    setMessages([])
    await refresh()
  }

  useEffect(() => {
    refresh()
  }, [refresh])

  return {
    status,
    messages,
    isLoading,
    isAuthRequired,
    error,
    refresh,
    lock,
    unlock,
    removeLock,
    clearMessages,
    /** Mark a message as read: updates local state immediately, and calls
     *  GET /messages/:id so the backend persists is_read=1 (it sets this as a
     *  side effect of fetching a single message). */
    markRead: useCallback((messageId: string) => {
      setMessages((prev) =>
        prev.map((m) => (m.id === messageId ? { ...m, is_read: true } : m))
      )
      // Also decrement unread count in status
      setStatus((prev) => {
        if (!prev) return prev
        return {
          ...prev,
          unread_messages: Math.max(0, prev.unread_messages - 1),
        }
      })

      const token = getToken()
      const headers: Record<string, string> = {}
      if (token) {
        headers['Authorization'] = `Bearer ${token}`
      }
      fetch(`/api/inboxes/${encodeURIComponent(inboxName)}/messages/${encodeURIComponent(messageId)}`, { headers }).catch(
        (e) => console.error('Failed to persist read status', e)
      )
    }, [inboxName, getToken]),
  }
}

