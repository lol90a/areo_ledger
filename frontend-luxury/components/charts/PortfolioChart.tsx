'use client'

import { PieChart, Pie, Cell, ResponsiveContainer, Legend, Tooltip } from 'recharts'

const data = [
  { name: 'Private Jets', value: 40, color: '#4578ff' },
  { name: 'Yachts', value: 30, color: '#6b94ff' },
  { name: 'Real Estate', value: 20, color: '#9bb7ff' },
  { name: 'Luxury Cars', value: 10, color: '#dbe7ff' },
]

export function PortfolioChart() {
  return (
    <ResponsiveContainer width="100%" height={300}>
      <PieChart>
        <Pie
          data={data}
          cx="50%"
          cy="50%"
          labelLine={false}
          outerRadius={100}
          fill="#8884d8"
          dataKey="value"
          label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
        >
          {data.map((entry, index) => (
            <Cell key={`cell-${index}`} fill={entry.color} />
          ))}
        </Pie>
        <Tooltip
          contentStyle={{
            backgroundColor: '#141414',
            border: '1px solid rgba(255,255,255,0.1)',
            borderRadius: '8px',
          }}
        />
        <Legend />
      </PieChart>
    </ResponsiveContainer>
  )
}

