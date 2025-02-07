import { Card, CardContent } from "@/components/ui/card"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from "recharts"

type OwnershipFraction = {
  memberId: string
  name: string
  avatar: string
  percentage: number
}

const COLORS = ["#0088FE", "#00C49F", "#FFBB28", "#FF8042", "#8884D8", "#83A6ED", "#8DD1E1", "#A4DE6C"]

export function OwnershipFractions({ fractions }: { fractions: OwnershipFraction[] }) {
  const data = fractions.map((fraction) => ({
    name: fraction.name,
    value: fraction.percentage,
  }))

  return (
    <Card className="bg-gray-800/30 border-gray-700 rounded-[2rem]">
      <CardContent className="p-6">
        <h3 className="text-xl font-semibold text-gray-200 mb-4">Ownership Fractions</h3>
        <div className="flex items-center justify-between">
          <div className="w-1/2">
            <ResponsiveContainer width="100%" height={200}>
              <PieChart>
                <Pie
                  data={data}
                  cx="50%"
                  cy="50%"
                  innerRadius={60}
                  outerRadius={80}
                  fill="#8884d8"
                  paddingAngle={5}
                  dataKey="value"
                >
                  {data.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip
                  formatter={(value: number) => `${value.toFixed(2)}%`}
                  contentStyle={{ background: "rgba(0, 0, 0, 0.8)", border: "none", borderRadius: "0.5rem" }}
                  itemStyle={{ color: "#fff" }}
                />
              </PieChart>
            </ResponsiveContainer>
          </div>
          <div className="w-1/2 space-y-2">
            {fractions.map((fraction, index) => (
              <div key={fraction.memberId} className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Avatar className="h-8 w-8">
                    <AvatarImage src={fraction.avatar} alt={fraction.name} />
                    <AvatarFallback>{fraction.name[0]}</AvatarFallback>
                  </Avatar>
                  <span className="text-sm font-medium text-gray-200">{fraction.name}</span>
                </div>
                <span className="text-sm font-medium text-gray-400" style={{ color: COLORS[index % COLORS.length] }}>
                  {fraction.percentage.toFixed(2)}%
                </span>
              </div>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

