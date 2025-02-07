export type Position = {
  position_id: string
  asset_symbol: string
  share_count: number
  share_percentage: number // basis points (100 = 1%)
  status: "active" | "pending_exit" | "closed"
  entry_timestamp: number
}

export type WindfallBalance = {
  shares: number
  share_percentage: number // basis points
  entry_price: number
  current_value: number
  unrealized_pnl: number
  realized_pnl: number
}

export type Proposal = {
  proposal_id: string
  proposal_type: "trade" | "actuator" | "emergency"
  votes_yes: number
  votes_no: number
  end_time: number
  executed: boolean
}

export type WalletInfo = {
  address: string
  ensName?: string
  avatar?: string
  balance: string
}

export type ProposalData = {
  asset_symbol: string
  size: number
  price: number
  is_entry: boolean
  description: string
}

export type FundMember = {
  id: string
  name: string
  avatar: string
  role: "admin" | "member"
}

export type Fund = {
  id: string
  name: string
  members: FundMember[]
}

export type ChatMessage = {
  id: string
  senderId: string
  content: string
  timestamp: number
}

