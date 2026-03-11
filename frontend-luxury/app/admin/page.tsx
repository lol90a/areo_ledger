'use client'

import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { StatCard } from '@/components/dashboard/StatCard'
import { Users, Briefcase, DollarSign, TrendingUp } from 'lucide-react'
import { formatCurrency } from '@/lib/utils'
import { useAdminStats, useBookings, usePayments } from '@/lib/hooks'

export default function AdminPage() {
  const [adminName, setAdminName] = useState('Admin')
  const { data: stats, isLoading: statsLoading } = useAdminStats()
  const { data: bookings, isLoading: bookingsLoading } = useBookings()
  const { data: payments, isLoading: paymentsLoading } = usePayments()

  useEffect(() => {
    // Get user data from localStorage
    const userStr = localStorage.getItem('user')
    if (userStr) {
      try {
        const user = JSON.parse(userStr)
        setAdminName(user.name || 'Admin')
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
        <h1 className="font-display text-4xl font-bold mb-2">Admin Dashboard</h1>
        <p className="text-muted">Welcome {adminName}, manage your platform and analytics</p>
      </motion.div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard
          title="Total Users"
          value={statsLoading ? '...' : stats?.users || 0}
          icon={Users}
          index={0}
        />
        <StatCard
          title="Total Bookings"
          value={statsLoading ? '...' : stats?.bookings || 0}
          icon={Briefcase}
          index={1}
        />
        <StatCard
          title="Total Revenue"
          value={statsLoading ? '...' : formatCurrency(stats?.revenue || 0)}
          icon={DollarSign}
          index={2}
        />
        <StatCard
          title="Active Assets"
          value={statsLoading ? '...' : stats?.flights || 0}
          icon={TrendingUp}
          index={3}
        />
      </div>

      {/* Recent Bookings */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="glass-card p-6 mb-8"
      >
        <h3 className="font-display text-xl font-bold mb-4">Recent Bookings</h3>
        {bookingsLoading ? (
          <div className="text-center py-8 text-muted">Loading...</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-white/10">
                  <th className="text-left p-3 text-sm font-semibold text-muted">Booking ID</th>
                  <th className="text-left p-3 text-sm font-semibold text-muted">Status</th>
                  <th className="text-right p-3 text-sm font-semibold text-muted">Amount</th>
                  <th className="text-left p-3 text-sm font-semibold text-muted">Date</th>
                </tr>
              </thead>
              <tbody>
                {bookings?.slice(0, 5).map((booking: any) => (
                  <tr key={booking.id} className="border-b border-white/10">
                    <td className="p-3 font-mono text-sm">{booking.id.substring(0, 8)}...</td>
                    <td className="p-3">
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
                        booking.status === 'confirmed' 
                          ? 'bg-green-500/20 text-green-500' 
                          : 'bg-yellow-500/20 text-yellow-500'
                      }`}>
                        {booking.status}
                      </span>
                    </td>
                    <td className="p-3 text-right font-semibold text-gold">
                      {formatCurrency(booking.total_price)}
                    </td>
                    <td className="p-3 text-sm text-muted">
                      {new Date(booking.created_at).toLocaleDateString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </motion.div>

      {/* Recent Payments */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
        className="glass-card p-6"
      >
        <h3 className="font-display text-xl font-bold mb-4">Recent Payments</h3>
        {paymentsLoading ? (
          <div className="text-center py-8 text-muted">Loading...</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-white/10">
                  <th className="text-left p-3 text-sm font-semibold text-muted">Payment ID</th>
                  <th className="text-left p-3 text-sm font-semibold text-muted">Chain</th>
                  <th className="text-left p-3 text-sm font-semibold text-muted">Token</th>
                  <th className="text-right p-3 text-sm font-semibold text-muted">Amount</th>
                  <th className="text-left p-3 text-sm font-semibold text-muted">Status</th>
                </tr>
              </thead>
              <tbody>
                {payments?.slice(0, 5).map((payment: any) => (
                  <tr key={payment.id} className="border-b border-white/10">
                    <td className="p-3 font-mono text-sm">{payment.id.substring(0, 8)}...</td>
                    <td className="p-3">{payment.chain}</td>
                    <td className="p-3 font-semibold">{payment.token}</td>
                    <td className="p-3 text-right font-semibold text-gold">
                      {formatCurrency(payment.amount)}
                    </td>
                    <td className="p-3">
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
                        payment.status === 'confirmed' 
                          ? 'bg-green-500/20 text-green-500' 
                          : 'bg-yellow-500/20 text-yellow-500'
                      }`}>
                        {payment.status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </motion.div>
    </div>
  )
}
