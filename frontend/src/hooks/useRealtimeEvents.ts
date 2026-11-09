import { useEffect } from 'react'

export interface RealtimeMailEvent {
  event: string
  id: string
  subject: string
  sender: string
}

export function useRealtimeEvents(
  inboxName: string,
  onNewEmail: (event: RealtimeMailEvent) => void
) {
  useEffect(() => {
    if (!inboxName) return

    const eventSource = new EventSource(`/api/inboxes/${encodeURIComponent(inboxName)}/events`)

    eventSource.onmessage = (event) => {
      try {
        const data: RealtimeMailEvent = JSON.parse(event.data)
        if (data.event === 'new_email') {
          onNewEmail(data)
        }
      } catch (err) {
        console.error('Failed to parse SSE event data', err)
      }
    }

    eventSource.onerror = (err) => {
      // EventSource automatically reconnects on error
      console.debug('SSE event source connection status changed', err)
    }

    return () => {
      eventSource.close()
    }
  }, [inboxName, onNewEmail])
}

