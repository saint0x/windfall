"use client"

import { useState } from "react"
import { motion } from "framer-motion"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { BarChart2, TrendingUp, Send } from "lucide-react"
import type { Fund } from "../../types"
import { usePositions } from "../../hooks/usePositions"
import { useChat } from "../../hooks/useChat"
import { Skeleton } from "@/components/ui/skeleton"
import { PerformanceGraph } from "./performance-graph"
import { JoinFund } from "../join-fund"
import { CreateFund } from "../create-fund"

// Mock data for funds and members
const mockFunds: Fund[] = [
  {
    id: "1",
    name: "Tech Growth Fund",
    members: [
      { id: "0", name: "You", avatar: "/placeholder.svg?height=32&width=32", role: "executor" },
      { id: "1", name: "Alice", avatar: "/placeholder.svg?height=32&width=32", role: "member" },
      { id: "2", name: "Bob", avatar: "/placeholder.svg?height=32&width=32", role: "member" },
      { id: "3", name: "Charlie", avatar: "/placeholder.svg?height=32&width=32", role: "member" },
    ],
  },
  {
    id: "2",
    name: "Crypto Ventures",
    members: [
      { id: "4", name: "David", avatar: "/placeholder.svg?height=32&width=32", role: "executor" },
      { id: "0", name: "You", avatar: "/placeholder.svg?height=32&width=32", role: "member" },
      { id: "5", name: "Eve", avatar: "/placeholder.svg?height=32&width=32", role: "member" },
    ],
  },
]

export function Dashboard({
  onViewChange,
  walletAddress,
}: {
  onViewChange: (
    view: "dashboard" | "position" | "governance" | "createFund" | "joinFund" | "manageFund",
    positionId?: string,
  ) => void
  walletAddress: string
}) {
  const { positions, loading, error } = usePositions(walletAddress)
  const [selectedFund, setSelectedFund] = useState<Fund>(mockFunds[0])
  const { messages, sendMessage } = useChat(selectedFund.id)
  const [newMessage, setNewMessage] = useState("")

  if (error) return <div className="text-red-500">Error: {error.message}</div>

  const handleSendMessage = (e: React.FormEvent) => {
    e.preventDefault()
    if (newMessage.trim()) {
      sendMessage(newMessage)
      setNewMessage("")
    }
  }

  return (
    <div className="p-8 min-h-screen">
      <div className="max-w-7xl mx-auto space-y-8">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-4xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500">
              Windfall
            </h1>
            <p className="text-gray-400 mt-1">Group Investment Dashboard</p>
          </div>
          <div className="flex gap-4">
            <CreateFund />
            <JoinFund />
            <Button
              variant="outline"
              className="bg-gray-800/50 border-gray-700 text-gray-200 hover:bg-gray-700/50 hover:text-white rounded-full"
              onClick={() => onViewChange("manageFund")}
            >
              Manage
              <span className="ml-2 text-xs bg-blue-500 text-white px-2 py-1 rounded-full">Executor</span>
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <Card className="bg-gray-800/30 border-gray-700 lg:col-span-2 rounded-[2rem]">
            <CardHeader>
              <CardTitle className="text-xl font-semibold text-gray-200">Active Positions</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {loading
                  ? Array(4)
                      .fill(0)
                      .map((_, index) => (
                        <Card key={index} className="bg-gray-800/50 border-gray-700 rounded-[1.5rem]">
                          <CardContent className="p-6">
                            <Skeleton className="h-6 w-24 bg-gray-700 mb-4 rounded-full" />
                            <Skeleton className="h-4 w-16 bg-gray-700 mb-6 rounded-full" />
                            <Skeleton className="h-8 w-32 bg-gray-700 rounded-full" />
                          </CardContent>
                        </Card>
                      ))
                  : positions.map((position) => (
                      <motion.div
                        key={position.position_id}
                        whileHover={{ scale: 1.03 }}
                        whileTap={{ scale: 0.98 }}
                        onClick={() => onViewChange("position", position.position_id)}
                      >
                        <Card className="bg-gray-800/50 border-gray-700 hover:bg-gray-700/50 transition-colors duration-200 cursor-pointer overflow-hidden group rounded-[1.5rem]">
                          <CardContent className="p-6">
                            <div className="flex justify-between items-start mb-4">
                              <div>
                                <h3 className="text-lg font-semibold text-gray-200">{position.asset_symbol}</h3>
                                <p className="text-sm text-gray-400">
                                  {(position.share_percentage / 100).toFixed(2)}% Share
                                </p>
                              </div>
                              <div className="w-10 h-10 rounded-full bg-gray-700/50 flex items-center justify-center group-hover:bg-blue-500/20 transition-colors duration-200">
                                <BarChart2 className="h-5 w-5 text-gray-400 group-hover:text-blue-400 transition-colors duration-200" />
                              </div>
                            </div>
                            <div className="space-y-2">
                              <div className="text-sm text-gray-400">Current Value</div>
                              <div className="text-2xl font-bold text-gray-200">$2,500.00</div>
                              <div className="text-sm text-emerald-400 flex items-center gap-1">
                                <TrendingUp className="h-4 w-4" />
                                +12.5%
                              </div>
                            </div>
                          </CardContent>
                        </Card>
                      </motion.div>
                    ))}
              </div>
              <Card className="bg-gray-800/50 border-gray-700 rounded-[1.5rem]">
                <CardHeader>
                  <CardTitle className="text-lg font-semibold text-gray-200">Portfolio Performance</CardTitle>
                </CardHeader>
                <CardContent>
                  <PerformanceGraph />
                </CardContent>
              </Card>
            </CardContent>
          </Card>

          <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
            <CardHeader>
              <CardTitle className="text-xl font-semibold text-gray-200">Fund Members & Chat</CardTitle>
            </CardHeader>
            <CardContent>
              <Tabs defaultValue={mockFunds[0].id} className="w-full">
                <TabsList className="grid w-full grid-cols-2 rounded-full">
                  {mockFunds.map((fund) => (
                    <TabsTrigger
                      key={fund.id}
                      value={fund.id}
                      onClick={() => setSelectedFund(fund)}
                      className="data-[state=active]:bg-gray-700/50 rounded-full"
                    >
                      {fund.name}
                    </TabsTrigger>
                  ))}
                </TabsList>
                {mockFunds.map((fund) => (
                  <TabsContent key={fund.id} value={fund.id} className="mt-4">
                    <div className="mb-4">
                      <h4 className="text-lg font-semibold text-gray-200 mb-2">Members</h4>
                      <div className="flex flex-wrap gap-2">
                        {fund.members.map((member) => (
                          <div
                            key={member.id}
                            className="flex items-center gap-2 bg-gray-700/30 rounded-full px-3 py-1"
                          >
                            <Avatar className="h-6 w-6">
                              <AvatarImage src={member.avatar} alt={member.name} />
                              <AvatarFallback>{member.name[0]}</AvatarFallback>
                            </Avatar>
                            <span className="text-sm text-gray-200">{member.name}</span>
                            {member.role === "executor" && (
                              <span className="text-xs text-blue-400 font-semibold">Executor</span>
                            )}
                          </div>
                        ))}
                      </div>
                    </div>
                    <div className="mt-4">
                      <h4 className="text-lg font-semibold text-gray-200 mb-2">Chat</h4>
                      <ScrollArea className="h-[300px] w-full rounded-[1.5rem] border border-gray-700 p-4">
                        {messages.map((message) => (
                          <div key={message.id} className="mb-4">
                            <div className="flex items-center gap-2 mb-1">
                              <Avatar className="h-6 w-6">
                                <AvatarImage
                                  src={fund.members.find((m) => m.id === message.senderId)?.avatar}
                                  alt={fund.members.find((m) => m.id === message.senderId)?.name}
                                />
                                <AvatarFallback>
                                  {fund.members.find((m) => m.id === message.senderId)?.name[0]}
                                </AvatarFallback>
                              </Avatar>
                              <span className="text-sm font-semibold text-gray-200">
                                {fund.members.find((m) => m.id === message.senderId)?.name}
                              </span>
                              <span className="text-xs text-gray-400">
                                {new Date(message.timestamp).toLocaleTimeString()}
                              </span>
                            </div>
                            <p className="text-sm text-gray-300 ml-8">{message.content}</p>
                          </div>
                        ))}
                      </ScrollArea>
                      <form onSubmit={handleSendMessage} className="mt-4 flex gap-2">
                        <Input
                          type="text"
                          placeholder="Type your message..."
                          value={newMessage}
                          onChange={(e) => setNewMessage(e.target.value)}
                          className="flex-grow bg-gray-700/50 border-gray-600 text-gray-200 rounded-full"
                        />
                        <Button type="submit" size="icon" className="bg-blue-500 hover:bg-blue-600 rounded-full">
                          <Send className="h-4 w-4" />
                        </Button>
                      </form>
                    </div>
                  </TabsContent>
                ))}
              </Tabs>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

