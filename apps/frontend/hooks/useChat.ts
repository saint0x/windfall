import { useState, useEffect } from "react"
import type { ChatMessage } from "../types"

export function useChat(fundId: string) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    async function fetchMessages() {
      try {
        // In a real application, you would fetch this data from an API
        // For now, we'll use mock data
        const mockMessages: ChatMessage[] = [
          {
            id: "1",
            senderId: "user1",
            content: "Hey team, what do you think about our latest investment?",
            timestamp: Date.now() - 3600000,
          },
          {
            id: "2",
            senderId: "user2",
            content: "I think it's performing well so far. The market seems bullish.",
            timestamp: Date.now() - 1800000,
          },
          {
            id: "3",
            senderId: "user3",
            content: "Agreed. Should we consider increasing our position?",
            timestamp: Date.now() - 900000,
          },
        ]

        setMessages(mockMessages)
        setLoading(false)
      } catch (err) {
        setError(err instanceof Error ? err : new Error("An error occurred while fetching messages"))
        setLoading(false)
      }
    }

    fetchMessages()
  }, [])

  const sendMessage = (content: string) => {
    const newMessage: ChatMessage = {
      id: Date.now().toString(),
      senderId: "currentUser", // In a real app, this would be the actual user's ID
      content,
      timestamp: Date.now(),
    }
    setMessages((prevMessages) => [...prevMessages, newMessage])
  }

  return { messages, loading, error, sendMessage }
}

