'use client'

import { motion } from 'framer-motion'
import { LucideIcon } from 'lucide-react'
import { formatPercentage } from '@/lib/utils'

interface StatCardProps {
  title: string
  value: string | number
  change?: number
  icon: LucideIcon
  index?: number
}

export function StatCard({ title, value, change, icon: Icon, index = 0 }: StatCardProps) {
  const isPositive = change !== undefined ? change >= 0 : undefined

  return (
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.08 }}
      whileHover={{ y: -2 }}
      className="interactive-card p-6"
    >
      <div className="mb-5 flex items-start justify-between">
        <div className="flex h-11 w-11 items-center justify-center rounded-xl border border-[rgba(var(--frame-strong),0.7)] bg-[rgb(var(--surface-highlight))]">
          <Icon className="h-5 w-5 text-white" />
        </div>
        {change !== undefined && (
          <span className={`text-xs font-medium ${isPositive ? 'text-[#7be0c7]' : 'text-red-400'}`}>
            {formatPercentage(change)}
          </span>
        )}
      </div>
      <div className="text-sm text-muted">{title}</div>
      <div className="mt-2 text-3xl font-semibold tracking-[-0.04em]">{value}</div>
    </motion.div>
  )
}
