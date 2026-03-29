'use client'

import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'

const data = [
  { month: 'Jan', value: 100000 },
  { month: 'Feb', value: 125000 },
  { month: 'Mar', value: 145000 },
  { month: 'Apr', value: 138000 },
  { month: 'May', value: 165000 },
  { month: 'Jun', value: 195000 },
]

export function PerformanceChart() {
  return (
    <ResponsiveContainer width="100%" height={300}>
      <LineChart data={data}>
        <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
        <XAxis dataKey="month" stroke="#9CA3AF" />
        <YAxis stroke="#9CA3AF" />
        <Tooltip
          contentStyle={{
            backgroundColor: '#141414',
            border: '1px solid rgba(255,255,255,0.1)',
            borderRadius: '8px',
          }}
        />
        <Line
          type="monotone"
          dataKey="value"
          stroke="#4578ff"
          strokeWidth={3}
          dot={{ fill: '#4578ff', r: 4 }}
        />
      </LineChart>
    </ResponsiveContainer>
  )
}


