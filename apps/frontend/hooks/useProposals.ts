import { useState, useEffect } from "react"
import type { Proposal } from "../types"

export function useProposals() {
  const [proposals, setProposals] = useState<Proposal[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    async function fetchProposals() {
      try {
        // In a real application, you would fetch this data from an API
        // For now, we'll use mock data
        const mockProposals: Proposal[] = [
          {
            proposal_id: "1",
            proposal_type: "trade",
            votes_yes: 10,
            votes_no: 5,
            end_time: Date.now() + 86400000, // 24 hours from now
            executed: false,
          },
          {
            proposal_id: "2",
            proposal_type: "actuator",
            votes_yes: 15,
            votes_no: 3,
            end_time: Date.now() + 172800000, // 48 hours from now
            executed: false,
          },
        ]

        setProposals(mockProposals)
        setLoading(false)
      } catch (err) {
        setError(err instanceof Error ? err : new Error("An error occurred while fetching proposals"))
        setLoading(false)
      }
    }

    fetchProposals()
  }, [])

  return { proposals, loading, error }
}

