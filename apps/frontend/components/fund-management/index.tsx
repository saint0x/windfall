"use client"

import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { ScrollArea } from "@/components/ui/scroll-area"
import { ArrowLeft, TrendingUp, Plus, Settings } from "lucide-react"
import { PerformanceGraph } from "../dashboard/performance-graph"
import { OwnershipFractions, type OwnershipFraction } from "./ownership-fractions"
import { useChat } from "../../hooks/useChat"
import type { Fund } from "../../types"

// Mock data for the fund
const mockFund: Fund = {
  id: "1",
  name: "Tech Growth Fund",
  members: [
    { id: "0", name: "You", avatar: "/placeholder.svg?height=32&width=32", role: "executor" },
    { id: "1", name: "Alice", avatar: "/placeholder.svg?height=32&width=32", role: "member" },
    { id: "2", name: "Bob", avatar: "/placeholder.svg?height=32&width=32", role: "member" },
    { id: "3", name: "Charlie", avatar: "/placeholder.svg?height=32&width=32", role: "member" },
  ],
}

const mockOwnershipFractions: OwnershipFraction[] = [
  { memberId: "0", name: "You", avatar: "/placeholder.svg?height=32&width=32", percentage: 40 },
  { memberId: "1", name: "Alice", avatar: "/placeholder.svg?height=32&width=32", percentage: 25 },
  { memberId: "2", name: "Bob", avatar: "/placeholder.svg?height=32&width=32", percentage: 20 },
  { memberId: "3", name: "Charlie", avatar: "/placeholder.svg?height=32&width=32", percentage: 15 },
]

export function FundManagement({
  onBack,
  walletAddress,
}: {
  onBack: () => void
  walletAddress: string
}) {
  const { messages, sendMessage } = useChat(mockFund.id)
  const [newMessage, setNewMessage] = useState("")

  const [activeTab, setActiveTab] = useState("overview")

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
          <div className="flex items-center gap-4">
            <Button variant="ghost" onClick={onBack} className="text-gray-400 hover:text-gray-200 rounded-full">
              <ArrowLeft className="h-4 w-4 mr-2" />
              Back to Dashboard
            </Button>
            <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500">
              {mockFund.name} Management
            </h1>
          </div>
          <Button
            variant="outline"
            className="bg-gray-800/50 border-gray-700 text-gray-200 hover:bg-gray-700/50 hover:text-white rounded-full"
          >
            <Settings className="mr-2 h-4 w-4" />
            Fund Settings
          </Button>
        </div>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full grid-cols-5 rounded-full bg-gray-800/30">
            <TabsTrigger value="overview" className="data-[state=active]:bg-gray-700/50 rounded-full">
              Overview
            </TabsTrigger>
            <TabsTrigger value="positions" className="data-[state=active]:bg-gray-700/50 rounded-full">
              Positions
            </TabsTrigger>
            <TabsTrigger value="proposals" className="data-[state=active]:bg-gray-700/50 rounded-full">
              Proposals
            </TabsTrigger>
            <TabsTrigger value="transactions" className="data-[state=active]:bg-gray-700/50 rounded-full">
              Transactions
            </TabsTrigger>
            <TabsTrigger value="members" className="data-[state=active]:bg-gray-700/50 rounded-full">
              Members
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="mt-6">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
              <Card className="bg-gray-800/30 border-gray-700 lg:col-span-2 rounded-[2rem]">
                <CardHeader>
                  <CardTitle className="text-xl font-semibold text-gray-200">Fund Performance</CardTitle>
                </CardHeader>
                <CardContent>
                  <PerformanceGraph />
                </CardContent>
              </Card>

              <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
                <CardHeader>
                  <CardTitle className="text-xl font-semibold text-gray-200">Fund Stats</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <p className="text-sm text-gray-400">Total Value</p>
                    <p className="text-2xl font-bold text-gray-200">$1,234,567.89</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-400">24h Change</p>
                    <p className="text-xl text-emerald-400 flex items-center gap-1">
                      <TrendingUp className="h-4 w-4" />
                      +5.67% ($12,345.67)
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-400">Number of Positions</p>
                    <p className="text-xl text-gray-200">{mockFund.members.length}</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-400">Number of Members</p>
                    <p className="text-xl text-gray-200">{mockFund.members.length}</p>
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="positions" className="mt-6">
            <div className="space-y-8">
              <OwnershipFractions fractions={mockOwnershipFractions} />
            </div>
          </TabsContent>

          <TabsContent value="proposals" className="mt-6">
            <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
              <CardHeader className="flex flex-row items-center justify-between">
                <CardTitle className="text-xl font-semibold text-gray-200">Active Proposals</CardTitle>
                <Button className="bg-blue-500 hover:bg-blue-600 text-white rounded-full">
                  <Plus className="mr-2 h-4 w-4" />
                  New Proposal
                </Button>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {/* Add mock proposals here */}
                  <Card className="bg-gray-800/50 border-gray-700 rounded-[1.5rem]">
                    <CardContent className="p-4">
                      <h3 className="text-lg font-semibold mb-2 text-gray-200">Proposal #1: Increase BTC position</h3>
                      <p className="text-sm text-gray-400 mb-2">Type: Trade</p>
                      <div className="flex justify-between items-center">
                        <div>
                          <p className="text-sm text-gray-300">Votes: Yes (3) / No (1)</p>
                          <p className="text-sm text-gray-400">Ends: 2023-07-15 23:59:59</p>
                        </div>
                        <div className="space-x-2">
                          <Button size="sm" variant="outline" className="rounded-full">
                            Details
                          </Button>
                          <Button size="sm" className="bg-green-500 hover:bg-green-600 text-white rounded-full">
                            Execute
                          </Button>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                  {/* Add more mock proposals as needed */}
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="transactions" className="mt-6">
            <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
              <CardHeader>
                <CardTitle className="text-xl font-semibold text-gray-200">Transaction History</CardTitle>
              </CardHeader>
              <CardContent>
                <ScrollArea className="h-[400px] w-full rounded-md border border-gray-700 p-4">
                  {/* Add mock transactions here */}
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="font-semibold text-gray-200">Buy BTC</p>
                        <p className="text-sm text-gray-400">2023-07-10 14:30:00</p>
                      </div>
                      <div className="text-right">
                        <p className="font-semibold text-emerald-400">+0.5 BTC</p>
                        <p className="text-sm text-gray-400">$15,000</p>
                      </div>
                    </div>
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="font-semibold text-gray-200">Sell ETH</p>
                        <p className="text-sm text-gray-400">2023-07-09 10:15:00</p>
                      </div>
                      <div className="text-right">
                        <p className="font-semibold text-red-400">-10 ETH</p>
                        <p className="text-sm text-gray-400">$18,500</p>
                      </div>
                    </div>
                    {/* Add more mock transactions as needed */}
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="members" className="mt-6">
            <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
              <CardHeader className="flex flex-row items-center justify-between">
                <CardTitle className="text-xl font-semibold text-gray-200">Fund Members</CardTitle>
                <Button className="bg-blue-500 hover:bg-blue-600 text-white rounded-full">
                  <Plus className="mr-2 h-4 w-4" />
                  Invite Member
                </Button>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {mockFund.members.map((member) => (
                    <div
                      key={member.id}
                      className="flex items-center justify-between bg-gray-800/50 p-4 rounded-[1.5rem]"
                    >
                      <div className="flex items-center gap-4">
                        <Avatar className="h-10 w-10">
                          <AvatarImage src={member.avatar} alt={member.name} />
                          <AvatarFallback>{member.name[0]}</AvatarFallback>
                        </Avatar>
                        <div>
                          <p className="font-semibold text-gray-200">{member.name}</p>
                          <p className="text-sm text-gray-400">{member.role}</p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <Button size="sm" variant="outline" className="rounded-full">
                          Edit
                        </Button>
                        {member.role !== "executor" && (
                          <Button size="sm" variant="destructive" className="rounded-full">
                            Remove
                          </Button>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}

