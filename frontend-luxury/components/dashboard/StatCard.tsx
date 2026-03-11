'use client'

import { motion } from 'framer-motion'
import { LucideIcon } from 'lucide-react'
import { formatCurrency, formatPercentage } from '@/lib/utils'

interface StatCardProps {
  title: string
  value: string | number
  change?: number
  icon: LucideIcon
  index?: number
}

export function StatCard({ title, value, change, icon: Icon, index = 0 }: StatCardProps) {
  const isPositive = change && change >= 0

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.1 }}
      className="glass-card p-6 hover:bg-white/10 transition-all"
    >
      <div className="flex items-start justify-between mb-4">
        <div className="p-3 rounded-lg bg-gold/20">
          <Icon className="h-6 w-6 text-gold" />
        </div>
        {change !== undefined && (
          <span className={`text-sm font-semibold ${isPositive ? 'text-green-500' : 'text-red-500'}`}>
            {formatPercentage(change)}
          </span>
        )}
      </div>
      <div className="text-sm text-muted mb-1">{title}</div>
      <div className="text-3xl font-bold font-display">{value}</div>
    </motion.div>
  )
}
