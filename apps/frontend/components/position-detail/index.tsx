"use client"

import { motion } from "framer-motion"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { ArrowLeft, PieChart, ArrowUpRight, ArrowDownRight } from "lucide-react"
import { usePositionValue } from "../../hooks/usePositionValue"
import { useWalletBalance } from "../../hooks/useWalletBalance"
import { Skeleton } from "@/components/ui/skeleton"

export function PositionDetail({ positionId, onBack }: { positionId: string; onBack: () => void }) {
  const { value, loading: valueLoading, error: valueError } = usePositionValue(positionId)
  const { balance, loading: balanceLoading, error: balanceError } = useWalletBalance("mock-wallet-address") // Replace with actual wallet address

  if (valueError) return <div className="text-red-500">Error: {valueError.message}</div>
  if (balanceError) return <div className="text-red-500">Error: {balanceError.message}</div>

  return (
    <div className="p-8 min-h-screen">
      <div className="max-w-7xl mx-auto space-y-8">
        <div className="flex items-center">
          <Button variant="ghost" onClick={onBack} className="mr-4 text-gray-400 hover:text-gray-200 rounded-full">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back
          </Button>
          <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500">
            Position Details
          </h1>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
            <CardHeader>
              <CardTitle className="text-xl font-semibold text-gray-200">Position Stats</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {valueLoading ? (
                <>
                  <Skeleton className="h-6 w-32 bg-gray-700 rounded-full" />
                  <Skeleton className="h-8 w-40 bg-gray-700 rounded-full" />
                  <Skeleton className="h-6 w-32 bg-gray-700 rounded-full" />
                  <Skeleton className="h-6 w-32 bg-gray-700 rounded-full" />
                </>
              ) : (
                <>
                  <div>
                    <p className="text-sm text-gray-400">Total Position Value</p>
                    <p className="text-2xl font-bold text-gray-200">${value.total_value.toFixed(2)}</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-400">Entry Price</p>
                    <p className="text-xl text-gray-300">${value.entry_price.toFixed(2)}</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-400">Current Price</p>
                    <p className="text-xl text-gray-300">${value.current_price.toFixed(2)}</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-400">PnL</p>
                    <div className="flex items-center">
                      <p className={`text-xl ${value.pnl >= 0 ? "text-green-400" : "text-red-400"}`}>
                        ${Math.abs(value.pnl).toFixed(2)}
                      </p>
                      {value.pnl >= 0 ? (
                        <ArrowUpRight className="h-4 w-4 text-green-400 ml-2" />
                      ) : (
                        <ArrowDownRight className="h-4 w-4 text-red-400 ml-2" />
                      )}
                    </div>
                  </div>
                </>
              )}
            </CardContent>
          </Card>

          <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
            <CardHeader>
              <CardTitle className="text-xl font-semibold text-gray-200">Share Distribution</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {balanceLoading ? (
                <>
                  <Skeleton className="h-32 w-32 rounded-full bg-gray-700 mx-auto" />
                  <Skeleton className="h-6 w-32 bg-gray-700 rounded-full" />
                  <Skeleton className="h-8 w-40 bg-gray-700 rounded-full" />
                </>
              ) : (
                <>
                  <div className="flex justify-center">
                    <PieChart className="h-32 w-32 text-blue-400" />
                  </div>
                  <div>
                    <p className="text-sm text-gray-400">Your Share</p>
                    <p className="text-2xl font-bold text-gray-200">{(balance.share_percentage / 100).toFixed(2)}%</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-400">Your Value</p>
                    <p className="text-xl text-gray-300">${balance.current_value.toFixed(2)}</p>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        </div>

        <div className="flex justify-center space-x-4">
          <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
            <Button
              variant="outline"
              className="bg-gray-800/50 border-gray-700 text-gray-200 hover:bg-gray-700/50 hover:text-white px-6 rounded-full"
            >
              Propose Exit
            </Button>
          </motion.div>
          <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
            <Button
              variant="outline"
              className="bg-gray-800/50 border-gray-700 text-gray-200 hover:bg-gray-700/50 hover:text-white px-6 rounded-full"
            >
              Transfer Shares
            </Button>
          </motion.div>
        </div>
      </div>
    </div>
  )
}

