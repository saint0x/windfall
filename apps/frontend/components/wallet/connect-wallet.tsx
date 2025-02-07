"use client"

import { useState } from "react"
import { X } from "lucide-react"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog"
import { ScrollArea } from "@/components/ui/scroll-area"
import { WalletOption } from "./wallet-option"
import { motion, AnimatePresence } from "framer-motion"
import type { WalletInfo } from "../../types"

const WALLETS = [
  {
    id: "metamask",
    name: "MetaMask",
    icon: "/placeholder.svg?height=48&width=48",
    description: "Connect with the most popular Web3 wallet",
    backgroundColor: "#F6851B",
  },
  {
    id: "aptos",
    name: "Aptos Wallet",
    icon: "/placeholder.svg?height=48&width=48",
    description: "Connect with your Aptos wallet",
    backgroundColor: "#2DD8A7",
  },
]

const dialogDescriptionId = "connect-wallet-dialog-description"

interface ConnectWalletProps {
  onConnect: (walletInfo: WalletInfo) => void
}

export function ConnectWallet({ onConnect }: ConnectWalletProps) {
  const [open, setOpen] = useState(false)

  const handleConnect = (walletId: string) => {
    // Simulate wallet connection
    const mockWalletInfo: WalletInfo = {
      address: "0x1234...5678",
      balance: "1000",
    }
    onConnect(mockWalletInfo)
    setOpen(false)
  }

  return (
    <>
      <motion.button
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        onClick={() => setOpen(true)}
        className="flex items-center gap-2 px-6 py-3 text-sm font-medium text-white bg-gradient-to-r from-blue-600 to-indigo-600 rounded-2xl hover:from-blue-700 hover:to-indigo-700 shadow-lg hover:shadow-xl transition-all duration-200"
      >
        Connect Wallet
      </motion.button>
      <AnimatePresence>
        {open && (
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogContent
              className="sm:max-w-[420px] bg-black/95 backdrop-blur-xl text-white border border-white/10 rounded-3xl shadow-2xl"
              aria-describedby={dialogDescriptionId}
            >
              <DialogHeader className="px-6 pt-6">
                <DialogTitle className="text-2xl font-bold bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
                  Connect Wallet
                </DialogTitle>
                <DialogDescription id={dialogDescriptionId} className="text-sm text-gray-400 mt-2">
                  Choose your preferred wallet to connect to Windfall
                </DialogDescription>
                <button
                  onClick={() => setOpen(false)}
                  className="absolute right-6 top-6 text-gray-400 hover:text-white transition-colors"
                >
                  <X className="h-4 w-4" />
                </button>
              </DialogHeader>
              <ScrollArea className="px-6 pb-6">
                <div className="space-y-4 py-4">
                  {WALLETS.map((wallet) => (
                    <WalletOption key={wallet.id} {...wallet} onConnect={() => handleConnect(wallet.id)} />
                  ))}
                </div>
              </ScrollArea>
            </DialogContent>
          </Dialog>
        )}
      </AnimatePresence>
    </>
  )
}

