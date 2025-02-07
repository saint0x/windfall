"use client"

import { useState } from "react"
import { ConnectWallet } from "./components/wallet/connect-wallet"
import { Dashboard } from "./components/dashboard"
import { PositionDetail } from "./components/position-detail"
import { Governance } from "./components/governance"
import { FundManagement } from "./components/fund-management"
import { JoinFund } from "./components/join-fund"
import type { WalletInfo } from "./types"
import { motion, AnimatePresence } from "framer-motion"

export default function App() {
  const [wallet, setWallet] = useState<WalletInfo | null>(null)
  const [currentView, setCurrentView] = useState<
    "dashboard" | "position" | "governance" | "createFund" | "joinFund" | "manageFund"
  >("dashboard")
  const [selectedPositionId, setSelectedPositionId] = useState<string | null>(null)
  const [pendingJoinRequests, setPendingJoinRequests] = useState<string[]>([])

  const handleConnectWallet = (walletInfo: WalletInfo) => {
    setWallet(walletInfo)
  }

  const handleViewChange = (
    view: "dashboard" | "position" | "governance" | "createFund" | "joinFund" | "manageFund",
    positionId?: string,
  ) => {
    setCurrentView(view)
    if (positionId) {
      setSelectedPositionId(positionId)
    }
  }

  const handleJoinFundRequest = (fundUsername: string) => {
    setPendingJoinRequests((prev) => [...prev, fundUsername])
  }

  return (
    <div className="w-full min-h-screen bg-gradient-to-br from-gray-900 to-black text-gray-200 overflow-auto">
      <AnimatePresence mode="wait">
        {wallet ? (
          <motion.div
            key="app"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.3 }}
          >
            {currentView === "dashboard" && (
              <Dashboard
                onViewChange={handleViewChange}
                walletAddress={wallet.address}
                pendingJoinRequests={pendingJoinRequests}
              />
            )}
            {currentView === "position" && selectedPositionId && (
              <PositionDetail positionId={selectedPositionId} onBack={() => handleViewChange("dashboard")} />
            )}
            {currentView === "governance" && <Governance onBack={() => handleViewChange("dashboard")} />}
            {currentView === "createFund" && (
              <div className="p-8">
                <h2 className="text-2xl font-bold mb-4">Create Fund</h2>
                <p>Create Fund functionality coming soon...</p>
              </div>
            )}
            {currentView === "joinFund" && (
              <JoinFund onJoinRequest={handleJoinFundRequest} onBack={() => handleViewChange("dashboard")} />
            )}
            {currentView === "manageFund" && (
              <FundManagement onBack={() => handleViewChange("dashboard")} walletAddress={wallet.address} />
            )}
          </motion.div>
        ) : (
          <motion.div
            key="welcome"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            transition={{ duration: 0.5 }}
            className="h-screen flex flex-col items-center justify-center p-8 bg-gradient-to-br from-gray-900 to-black"
          >
            <h1 className="text-5xl font-bold mb-6 bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500">
              Welcome to Windfall
            </h1>
            <p className="text-gray-400 mb-10 text-center max-w-md text-lg">
              Connect your wallet to start managing your group investments with style
            </p>
            <ConnectWallet onConnect={handleConnectWallet} />
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

