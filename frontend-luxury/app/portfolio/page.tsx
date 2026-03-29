'use client'

import { useEffect, useState } from 'react'
import { motion } from 'framer-motion'
import { formatCurrency } from '@/lib/utils'
import { TrendingUp, Loader2 } from 'lucide-react'
import { api } from '@/lib/api'

export default function PortfolioPage() {
  const [portfolioData, setPortfolioData] = useState<any>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const fetchPortfolio = async () => {
      try {
        const userId = localStorage.getItem('userId') || 'guest'
        const response = await api.portfolio.getUserPortfolio(userId)
        setPortfolioData(response.data)
      } catch (error) {
        console.error('Failed to fetch portfolio:', error)
      } finally {
        setLoading(false)
      }
    }
    fetchPortfolio()
  }, [])

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center min-h-screen">
        <Loader2 className="h-12 w-12 animate-spin text-white" />
      </div>
    )
  }

  const portfolioAssets = portfolioData?.assets || []
  const totalPurchase = portfolioData?.totalValue || 0
  const totalCurrentValue = portfolioData?.totalValue || 0
  const totalAssets = portfolioAssets.length

  return (
    <div className="p-8">
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="mb-8"
      >
        <h1 className="font-display text-4xl font-bold mb-2">My Assets</h1>
        <p className="text-muted">Track your luxury asset purchases</p>
      </motion.div>

      {/* Summary Cards */}
      <div className="grid md:grid-cols-3 gap-6 mb-8">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="glass-card p-6"
        >
          <div className="text-sm text-muted mb-2">Total Purchase Value</div>
          <div className="text-3xl font-bold font-display">{formatCurrency(totalPurchase)}</div>
        </motion.div>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="glass-card p-6"
        >
          <div className="text-sm text-muted mb-2">Current Value</div>
          <div className="text-3xl font-bold font-display text-white">{formatCurrency(totalCurrentValue)}</div>
        </motion.div>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="glass-card p-6"
        >
          <div className="text-sm text-muted mb-2">Total Assets</div>
          <div className="text-3xl font-bold font-display text-green-500 flex items-center">
            <TrendingUp className="h-6 w-6 mr-2" />
            {totalAssets}
          </div>
        </motion.div>
      </div>

      {/* Portfolio Table */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="glass-card overflow-hidden"
      >
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-[rgba(var(--accent),0.30)]">
                <th className="text-left p-4 text-sm font-semibold text-muted">Asset</th>
                <th className="text-left p-4 text-sm font-semibold text-muted">Type</th>
                <th className="text-right p-4 text-sm font-semibold text-muted">Quantity</th>
                <th className="text-right p-4 text-sm font-semibold text-muted">Purchase Price</th>
                <th className="text-right p-4 text-sm font-semibold text-muted">Current Value</th>
                <th className="text-right p-4 text-sm font-semibold text-muted">Status</th>
              </tr>
            </thead>
            <tbody>
              {portfolioAssets.length === 0 ? (
                <tr>
                  <td colSpan={6} className="p-8 text-center text-muted">
                    No assets in your portfolio yet. Start investing!
                  </td>
                </tr>
              ) : (
                portfolioAssets.map((asset: any, index: number) => (
                  <motion.tr
                    key={asset.assetId}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: 0.5 + index * 0.1 }}
                    className="border-b border-[rgba(var(--accent),0.30)] hover:bg-white/5 transition-colors"
                  >
                    <td className="p-4">
                      <div className="font-semibold">{asset.assetName}</div>
                    </td>
                    <td className="p-4">
                      <span className="px-3 py-1 bg-[rgba(var(--accent),0.20)] text-white rounded-full text-xs">
                        {asset.assetType}
                      </span>
                    </td>
                    <td className="p-4 text-right font-semibold">{asset.tokensOwned}</td>
                    <td className="p-4 text-right">{formatCurrency(asset.investmentValue)}</td>
                    <td className="p-4 text-right font-semibold text-white">
                      {formatCurrency(asset.currentValue)}
                    </td>
                    <td className="p-4 text-right">
                      <div className="flex items-center justify-end font-semibold text-green-500">
                        <TrendingUp className="h-4 w-4 mr-1" />
                        +{asset.roi.toFixed(2)}%
                      </div>
                    </td>
                  </motion.tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </motion.div>
    </div>
  )
}



