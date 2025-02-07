"use client"

import { Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts"
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"

const data = [
  { date: "Jan 1", value: 1000 },
  { date: "Feb 1", value: 1200 },
  { date: "Mar 1", value: 1100 },
  { date: "Apr 1", value: 1300 },
  { date: "May 1", value: 1500 },
  { date: "Jun 1", value: 1400 },
  { date: "Jul 1", value: 1600 },
]

export function PerformanceGraph() {
  return (
    <ChartContainer
      config={{
        value: {
          label: "Portfolio Value",
          color: "hsl(142, 76%, 36%)", // Explicit green color
        },
      }}
      className="h-[320px] w-full"
    >
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data} margin={{ top: 16, right: 24, left: 16, bottom: 24 }}>
          <XAxis
            dataKey="date"
            stroke="hsl(var(--muted-foreground))"
            fontSize={12}
            tickLine={false}
            axisLine={false}
            dy={8} // Add some padding between axis and labels
          />
          <YAxis
            stroke="hsl(var(--muted-foreground))"
            fontSize={12}
            tickLine={false}
            axisLine={false}
            tickFormatter={(value) => `$${value}`}
            dx={-4} // Add some padding between axis and labels
          />
          <Tooltip content={<ChartTooltipContent />} />
          <Line type="monotone" dataKey="value" stroke="hsl(142, 76%, 36%)" strokeWidth={2} dot={false} />
        </LineChart>
      </ResponsiveContainer>
    </ChartContainer>
  )
}

