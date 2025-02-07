"use client"

import { useState } from "react"
import { motion } from "framer-motion"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { AlertCircle, Loader2 } from "lucide-react"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export function JoinFund() {
  const [isOpen, setIsOpen] = useState(false)
  const [fundId, setFundId] = useState("")
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [pendingRequests, setPendingRequests] = useState<string[]>([])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsSubmitting(true)
    setError(null)

    // Simulate API call
    await new Promise((resolve) => setTimeout(resolve, 1500))

    if (fundId.toLowerCase() === "invalid") {
      setError("Invalid fund ID. Please try again.")
      setIsSubmitting(false)
    } else {
      setPendingRequests((prev) => [...prev, fundId])
      setIsSubmitting(false)
      setIsOpen(false)
      setFundId("")
    }
  }

  return (
    <>
      <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
        <Button
          variant="outline"
          className="bg-gray-800/50 border-gray-700 text-gray-200 hover:bg-gray-700/50 hover:text-white rounded-full"
          onClick={() => setIsOpen(true)}
        >
          Join Fund
        </Button>
      </motion.div>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="sm:max-w-[425px] bg-gray-800 border-gray-700 rounded-[2rem] p-6 shadow-lg">
          <DialogHeader className="pb-4">
            <DialogTitle className="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500">
              Join a Fund
            </DialogTitle>
            <DialogDescription className="text-sm text-gray-400">
              Enter the unique ID of the fund you want to join.
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="fundId" className="text-sm font-medium text-gray-200">
                Fund ID
              </Label>
              <Input
                id="fundId"
                value={fundId}
                onChange={(e) => setFundId(e.target.value)}
                className="bg-gray-700 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-[1.5rem]"
                placeholder="Enter fund ID"
                required
              />
            </div>
            {error && (
              <Alert variant="destructive" className="bg-red-900/50 border-red-800 text-red-200 rounded-[1.5rem]">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <div className="flex justify-end">
              <Button
                type="submit"
                disabled={isSubmitting}
                className="bg-blue-500 hover:bg-blue-600 text-white rounded-[1.5rem]"
              >
                {isSubmitting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Submitting
                  </>
                ) : (
                  "Submit Join Request"
                )}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {pendingRequests.length > 0 && (
        <Card className="mt-4 bg-gray-800 border-gray-700 rounded-[2rem] shadow-lg">
          <CardHeader>
            <CardTitle className="text-xl font-semibold text-gray-200">Pending Join Requests</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2">
              {pendingRequests.map((request, index) => (
                <li key={index} className="bg-gray-700 border border-gray-600 rounded-[1.5rem] px-4 py-2 text-gray-200">
                  Fund ID: {request} - Waiting for approval
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}
    </>
  )
}

