export interface InboxStatus {
  name: string
  is_locked: boolean
  total_messages: number
  unread_messages: number
  created_at: string
  last_active_at: string
}

export interface Message {
  id: string
  inbox_name: string
  sender: string
  recipient: string
  subject: string
  snippet: string
  body_text?: string
  body_html?: string
  raw_headers: string
  detected_otp?: string
  detected_links?: string // JSON array
  is_read: boolean
  is_favorite: boolean
  is_deleted: boolean
  received_at: string
}

export interface Attachment {
  id: string
  message_id: string
  filename: string
  content_type: string
  size_bytes: number
  disk_path: string
  created_at: string
}

