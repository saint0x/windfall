"use client"

import { useState } from "react"
import { motion } from "framer-motion"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { useCreateProposal } from "../../hooks/useCreateProposal"
import { Textarea } from "@/components/ui/textarea"
import { Label } from "@/components/ui/label"

export function CreateProposal({ onClose }: { onClose: () => void }) {
  const [proposalData, setProposalData] = useState({
    asset_symbol: "",
    size: 0,
    price: 0,
    is_entry: true,
    description: "",
  })

  const { createProposal, loading, error } = useCreateProposal()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createProposal(proposalData)
      onClose()
    } catch (error) {
      console.error("Error creating proposal:", error)
    }
  }

  return (
    <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
      <CardHeader>
        <CardTitle className="text-xl font-semibold text-gray-200">Create Proposal</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="asset_symbol" className="text-sm font-medium text-gray-300">
              Asset Symbol
            </Label>
            <Input
              id="asset_symbol"
              value={proposalData.asset_symbol}
              onChange={(e) => setProposalData({ ...proposalData, asset_symbol: e.target.value })}
              className="bg-gray-700/50 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-full"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="size" className="text-sm font-medium text-gray-300">
              Position Size
            </Label>
            <Input
              id="size"
              type="number"
              value={proposalData.size}
              onChange={(e) => setProposalData({ ...proposalData, size: Number.parseFloat(e.target.value) })}
              className="bg-gray-700/50 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-full"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="price" className="text-sm font-medium text-gray-300">
              Price
            </Label>
            <Input
              id="price"
              type="number"
              value={proposalData.price}
              onChange={(e) => setProposalData({ ...proposalData, price: Number.parseFloat(e.target.value) })}
              className="bg-gray-700/50 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-full"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="is_entry" className="text-sm font-medium text-gray-300">
              Entry/Exit
            </Label>
            <Select
              value={proposalData.is_entry ? "entry" : "exit"}
              onValueChange={(value) => setProposalData({ ...proposalData, is_entry: value === "entry" })}
            >
              <SelectTrigger
                id="is_entry"
                className="bg-gray-700/50 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-full"
              >
                <SelectValue placeholder="Select entry or exit" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="entry">Entry</SelectItem>
                <SelectItem value="exit">Exit</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="description" className="text-sm font-medium text-gray-300">
              Description
            </Label>
            <Textarea
              id="description"
              value={proposalData.description}
              onChange={(e) => setProposalData({ ...proposalData, description: e.target.value })}
              className="bg-gray-700/50 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-[1.5rem]"
              required
            />
          </div>
          <div className="flex justify-end space-x-4">
            <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
              <Button
                type="button"
                variant="outline"
                onClick={onClose}
                className="bg-gray-700/50 text-gray-200 hover:bg-gray-600/50 border-gray-600 rounded-full"
              >
                Cancel
              </Button>
            </motion.div>
            <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
              <Button
                type="submit"
                disabled={loading}
                className="bg-blue-500 text-white hover:bg-blue-600 focus:ring-blue-500 rounded-full"
              >
                {loading ? "Creating..." : "Create Proposal"}
              </Button>
            </motion.div>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}

