'use client'

import { useState, useEffect } from 'react'
import { StatCard } from '@/components/dashboard/StatCard'
import { PortfolioChart } from '@/components/charts/PortfolioChart'
import { PerformanceChart } from '@/components/charts/PerformanceChart'
import { TrendingUp, Briefcase, DollarSign, Percent } from 'lucide-react'
import { motion } from 'framer-motion'

export default function DashboardPage() {
  const [userName, setUserName] = useState('User')

  useEffect(() => {
    // Get user data from localStorage
    const userStr = localStorage.getItem('user')
    if (userStr) {
      try {
        const user = JSON.parse(userStr)
        setUserName(user.name || 'User')
      } catch (error) {
        console.error('Error parsing user data:', error)
      }
    }
  }, [])
  return (
    <div className="p-8">
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="mb-8"
      >
        <h1 className="font-display text-4xl font-bold mb-2">My Dashboard</h1>
        <p className="text-muted">Welcome back, {userName}. Here's your purchase overview.</p>
      </motion.div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard
          title="Total Spent"
          value="$2.4M"
          change={12.5}
          icon={DollarSign}
          index={0}
        />
        <StatCard
          title="Assets Owned"
          value="12"
          change={3}
          icon={Briefcase}
          index={1}
        />
        <StatCard
          title="Flights Booked"
          value="18"
          change={2.1}
          icon={Percent}
          index={2}
        />
        <StatCard
          title="This Month"
          value="$24K"
          change={8.3}
          icon={TrendingUp}
          index={3}
        />
      </div>

      {/* Charts Grid */}
      <div className="grid lg:grid-cols-2 gap-6 mb-8">
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.4 }}
          className="glass-card p-6"
        >
          <h3 className="font-display text-xl font-bold mb-4">Asset Distribution</h3>
          <PortfolioChart />
        </motion.div>

        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.5 }}
          className="glass-card p-6"
        >
          <h3 className="font-display text-xl font-bold mb-4">Purchase History</h3>
          <PerformanceChart />
        </motion.div>
      </div>

      {/* Recent Activity */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
        className="glass-card p-6"
      >
        <h3 className="font-display text-xl font-bold mb-4">Recent Activity</h3>
        <div className="space-y-4">
          {[
            { action: 'Purchased', asset: 'Gulfstream G650', amount: '$50,000', time: '2 hours ago' },
            { action: 'Booked', asset: 'Miami to London Flight', amount: '$25,000', time: '1 day ago' },
            { action: 'Purchased', asset: 'Manhattan Penthouse', amount: '$3.2M', time: '3 days ago' },
          ].map((activity, index) => (
            <div
              key={index}
              className="flex items-center justify-between py-3 border-b border-[rgba(var(--accent),0.30)] last:border-0"
            >
              <div>
                <div className="font-semibold">{activity.action} - {activity.asset}</div>
                <div className="text-sm text-muted">{activity.time}</div>
              </div>
              <div className="text-white font-semibold">{activity.amount}</div>
            </div>
          ))}
        </div>
      </motion.div>
    </div>
  )
}



