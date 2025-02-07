"use client"

import { useState } from "react"
import { motion } from "framer-motion"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { ArrowLeft, Plus } from "lucide-react"
import { useProposals } from "../../hooks/useProposals"
import { CreateProposal } from "./create-proposal"
import { Skeleton } from "@/components/ui/skeleton"

export function Governance({ onBack }: { onBack: () => void }) {
  const { proposals, loading, error } = useProposals()
  const [showCreateProposal, setShowCreateProposal] = useState(false)

  if (error) return <div className="text-red-500">Error: {error.message}</div>

  return (
    <div className="p-8 min-h-screen">
      <div className="max-w-7xl mx-auto space-y-8">
        <div className="flex items-center justify-between">
          <div className="flex items-center">
            <Button variant="ghost" onClick={onBack} className="mr-4 text-gray-400 hover:text-gray-200 rounded-full">
              <ArrowLeft className="h-4 w-4 mr-2" />
              Back
            </Button>
            <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500">
              Governance
            </h1>
          </div>
          <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
            <Button
              variant="outline"
              className="bg-gray-800/50 border-gray-700 text-gray-200 hover:bg-gray-700/50 hover:text-white px-6 rounded-full"
              onClick={() => setShowCreateProposal(true)}
            >
              <Plus className="mr-2 h-4 w-4" />
              Create Proposal
            </Button>
          </motion.div>
        </div>

        {showCreateProposal ? (
          <CreateProposal onClose={() => setShowCreateProposal(false)} />
        ) : (
          <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
            <CardHeader>
              <CardTitle className="text-xl font-semibold text-gray-200">Active Proposals</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {loading
                  ? Array(3)
                      .fill(0)
                      .map((_, index) => (
                        <Card key={index} className="bg-gray-800/50 border-gray-700 rounded-[1.5rem]">
                          <CardContent className="p-4">
                            <Skeleton className="h-6 w-32 bg-gray-700 mb-2 rounded-full" />
                            <Skeleton className="h-4 w-24 bg-gray-700 mb-4 rounded-full" />
                            <div className="flex justify-between items-center">
                              <div>
                                <Skeleton className="h-4 w-40 bg-gray-700 mb-2 rounded-full" />
                                <Skeleton className="h-4 w-32 bg-gray-700 rounded-full" />
                              </div>
                              <div className="space-x-2">
                                <Skeleton className="h-8 w-20 bg-gray-700 inline-block rounded-full" />
                                <Skeleton className="h-8 w-20 bg-gray-700 inline-block rounded-full" />
                              </div>
                            </div>
                          </CardContent>
                        </Card>
                      ))
                  : proposals.map((proposal) => (
                      <Card key={proposal.proposal_id} className="bg-gray-800/50 border-gray-700 rounded-[1.5rem]">
                        <CardContent className="p-4">
                          <h3 className="text-lg font-semibold mb-2 text-gray-200">Proposal #{proposal.proposal_id}</h3>
                          <p className="text-sm text-gray-400 mb-2">Type: {proposal.proposal_type}</p>
                          <div className="flex justify-between items-center">
                            <div>
                              <p className="text-sm text-gray-300">
                                Votes: Yes ({proposal.votes_yes}) / No ({proposal.votes_no})
                              </p>
                              <p className="text-sm text-gray-400">
                                Ends: {new Date(proposal.end_time).toLocaleString()}
                              </p>
                            </div>
                            <div className="space-x-2">
                              <Button
                                variant="outline"
                                size="sm"
                                className="bg-green-500/20 text-green-400 hover:bg-green-500/30 border-green-500/30 rounded-full"
                              >
                                Vote Yes
                              </Button>
                              <Button
                                variant="outline"
                                size="sm"
                                className="bg-red-500/20 text-red-400 hover:bg-red-500/30 border-red-500/30 rounded-full"
                              >
                                Vote No
                              </Button>
                            </div>
                          </div>
                        </CardContent>
                      </Card>
                    ))}
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  )
}

