"use client"

import { useState } from "react"
import { motion } from "framer-motion"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Switch } from "@/components/ui/switch"
import { Loader2, Plus, X } from "lucide-react"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"

interface CreateFundFormData {
  name: string
  description: string
  initialInvestment: string
  strategy: "conservative" | "moderate" | "aggressive"
  isPrivate: boolean
  invitedMembers: string[]
}

const initialFormData: CreateFundFormData = {
  name: "",
  description: "",
  initialInvestment: "",
  strategy: "moderate",
  isPrivate: true,
  invitedMembers: [],
}

export function CreateFund() {
  const [isOpen, setIsOpen] = useState(false)
  const [formData, setFormData] = useState<CreateFundFormData>(initialFormData)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [newMember, setNewMember] = useState("")

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsSubmitting(true)
    setError(null)

    try {
      // Simulate API call
      await new Promise((resolve) => setTimeout(resolve, 1500))

      console.log("Creating fund with data:", formData)

      setIsOpen(false)
      setFormData(initialFormData)
    } catch (err) {
      setError("Failed to create fund. Please try again.")
    } finally {
      setIsSubmitting(false)
    }
  }

  const addMember = () => {
    if (newMember && !formData.invitedMembers.includes(newMember)) {
      setFormData({
        ...formData,
        invitedMembers: [...formData.invitedMembers, newMember],
      })
      setNewMember("")
    }
  }

  const removeMember = (member: string) => {
    setFormData({
      ...formData,
      invitedMembers: formData.invitedMembers.filter((m) => m !== member),
    })
  }

  return (
    <>
      <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
        <Button
          variant="outline"
          className="bg-gray-800/50 border-gray-700 text-gray-200 hover:bg-gray-700/50 hover:text-white rounded-full"
          onClick={() => setIsOpen(true)}
        >
          <Plus className="mr-2 h-4 w-4" />
          Create Fund
        </Button>
      </motion.div>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="sm:max-w-[525px] bg-gray-800 border-gray-700 rounded-[2rem] p-6 shadow-lg">
          <DialogHeader className="pb-4">
            <DialogTitle className="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500">
              Create New Fund
            </DialogTitle>
            <DialogDescription className="text-sm text-gray-400">
              Set up your investment fund and define its parameters.
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Fund Name */}
            <div className="space-y-2">
              <Label htmlFor="name" className="text-sm font-medium text-gray-200">
                Fund Name
              </Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="bg-gray-700 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-[1.5rem]"
                placeholder="Enter fund name"
                required
              />
            </div>

            {/* Description */}
            <div className="space-y-2">
              <Label htmlFor="description" className="text-sm font-medium text-gray-200">
                Description
              </Label>
              <Textarea
                id="description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                className="bg-gray-700 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-[1.5rem] min-h-[100px]"
                placeholder="Describe your fund's objectives and strategy"
                required
              />
            </div>

            {/* Initial Investment */}
            <div className="space-y-2">
              <Label htmlFor="initialInvestment" className="text-sm font-medium text-gray-200">
                Initial Investment (USD)
              </Label>
              <Input
                id="initialInvestment"
                type="number"
                min="0"
                step="0.01"
                value={formData.initialInvestment}
                onChange={(e) => setFormData({ ...formData, initialInvestment: e.target.value })}
                className="bg-gray-700 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-[1.5rem]"
                placeholder="Enter initial investment amount"
                required
              />
            </div>

            {/* Investment Strategy */}
            <div className="space-y-2">
              <Label className="text-sm font-medium text-gray-200">Investment Strategy</Label>
              <Tabs
                value={formData.strategy}
                onValueChange={(value: "conservative" | "moderate" | "aggressive") =>
                  setFormData({ ...formData, strategy: value })
                }
              >
                <TabsList className="grid w-full grid-cols-3 rounded-[1.5rem] bg-gray-700 p-1">
                  <TabsTrigger value="conservative" className="rounded-[1.25rem] data-[state=active]:bg-blue-500">
                    Conservative
                  </TabsTrigger>
                  <TabsTrigger value="moderate" className="rounded-[1.25rem] data-[state=active]:bg-blue-500">
                    Moderate
                  </TabsTrigger>
                  <TabsTrigger value="aggressive" className="rounded-[1.25rem] data-[state=active]:bg-blue-500">
                    Aggressive
                  </TabsTrigger>
                </TabsList>
              </Tabs>
            </div>

            {/* Private Fund Toggle */}
            <div className="flex items-center justify-between rounded-[1.5rem] bg-gray-700 p-4">
              <Label htmlFor="isPrivate" className="text-sm font-medium text-gray-200">
                Private Fund
              </Label>
              <Switch
                id="isPrivate"
                checked={formData.isPrivate}
                onCheckedChange={(checked) => setFormData({ ...formData, isPrivate: checked })}
              />
            </div>

            {/* Invite Members */}
            <div className="space-y-2">
              <Label className="text-sm font-medium text-gray-200">Invite Members</Label>
              <div className="flex space-x-2">
                <Input
                  value={newMember}
                  onChange={(e) => setNewMember(e.target.value)}
                  className="bg-gray-700 border-gray-600 text-gray-200 focus:ring-blue-500 focus:border-blue-500 rounded-[1.5rem] flex-grow"
                  placeholder="Enter member's email or username"
                />
                <Button
                  type="button"
                  onClick={addMember}
                  className="bg-blue-500 hover:bg-blue-600 text-white rounded-[1.5rem]"
                >
                  Add
                </Button>
              </div>
              <div className="space-y-2 mt-2">
                {formData.invitedMembers.map((member) => (
                  <div key={member} className="flex items-center justify-between bg-gray-700 rounded-[1.5rem] p-2">
                    <span className="text-gray-200">{member}</span>
                    <Button
                      type="button"
                      onClick={() => removeMember(member)}
                      variant="ghost"
                      size="sm"
                      className="text-gray-400 hover:text-gray-200"
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                ))}
              </div>
            </div>

            {/* Error Alert */}
            {error && (
              <Alert variant="destructive" className="bg-red-900/50 border-red-800 text-red-200 rounded-[1.5rem]">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            {/* Submit Button */}
            <div className="flex justify-end pt-4">
              <Button
                type="submit"
                disabled={isSubmitting}
                className="bg-blue-500 hover:bg-blue-600 text-white rounded-[1.5rem] px-8"
              >
                {isSubmitting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Creating Fund...
                  </>
                ) : (
                  "Create Fund"
                )}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
    </>
  )
}

