"use client"

import { motion } from "framer-motion"

interface WalletOptionProps {
  id: string
  name: string
  icon: string
  description: string
  backgroundColor: string
  onConnect: () => void
}

export function WalletOption({ id, name, icon, description, backgroundColor, onConnect }: WalletOptionProps) {
  return (
    <motion.button
      whileHover={{ scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
      className="group relative w-full p-6 rounded-2xl bg-white/5 hover:bg-white/10 transition-all duration-200 overflow-hidden"
      onClick={onConnect}
    >
      <div className="absolute inset-0 bg-gradient-to-r from-transparent to-white/5 opacity-0 group-hover:opacity-100 transition-opacity duration-200" />
      <div className="relative flex items-center gap-4">
        <div className="w-12 h-12 rounded-2xl flex items-center justify-center" style={{ backgroundColor }}>
          <img src={icon || "/placeholder.svg"} alt={name} className="w-8 h-8" />
        </div>
        <div className="text-left flex-1">
          <h3 className="font-semibold text-lg">{name}</h3>
          <p className="text-sm text-gray-400 mt-1">{description}</p>
        </div>
        <div className="w-8 h-8 rounded-full bg-white/5 group-hover:bg-white/10 flex items-center justify-center transition-colors duration-200">
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            className="text-white/50 group-hover:text-white transition-colors duration-200"
          >
            <path
              d="M6 12L10 8L6 4"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>
      </div>
    </motion.button>
  )
}

