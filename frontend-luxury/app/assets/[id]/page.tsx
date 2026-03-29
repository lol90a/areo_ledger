'use client'

import { useState } from 'react'
import { Navbar } from '@/components/navbar/Navbar'
import { Button } from '@/components/ui/button'
import { motion } from 'framer-motion'
import { formatCurrency, formatNumber } from '@/lib/utils'
import { ArrowLeft, TrendingUp, Users, Clock, MapPin } from 'lucide-react'
import Link from 'next/link'
import Image from 'next/image'

export default function AssetDetailPage({ params }: { params: { id: string } }) {
  const [tokenAmount, setTokenAmount] = useState(1)

  // Mock data - replace with API call
  const asset = {
    id: params.id,
    name: 'Gulfstream G650',
    type: 'Private Jet',
    image: 'https://images.unsplash.com/photo-1540962351504-03099e0a754b?w=1200',
    totalValue: 65000000,
    tokenPrice: 10000,
    availableTokens: 3250,
    totalTokens: 6500,
    roi: 18.5,
    location: 'Miami, FL',
    description: 'The Gulfstream G650 represents the pinnacle of private aviation. With its ultra-long-range capabilities and luxurious interior, this aircraft offers unparalleled comfort and performance.',
    specifications: {
      'Max Range': '7,000 nm',
      'Max Speed': 'Mach 0.925',
      'Passengers': '19',
      'Year': '2023',
      'Condition': 'Excellent',
    },
  }

  const purchaseTotal = tokenAmount * asset.tokenPrice

  return (
    <div className="min-h-screen">
      <Navbar />
      
      <main className="pt-32 pb-20 px-4">
        <div className="container mx-auto">
          {/* Back Button */}
          <Link href="/assets">
            <Button variant="ghost" className="mb-8">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to Marketplace
            </Button>
          </Link>

          <div className="grid lg:grid-cols-3 gap-8">
            {/* Left Column - Asset Details */}
            <div className="lg:col-span-2 space-y-8">
              {/* Image Gallery */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                className="relative h-96 rounded-xl overflow-hidden"
              >
                <Image
                  src={asset.image}
                  alt={asset.name}
                  fill
                  className="object-cover"
                />
              </motion.div>

              {/* Asset Info */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.1 }}
                className="glass-card p-8"
              >
                <div className="flex items-start justify-between mb-6">
                  <div>
                    <h1 className="font-display text-4xl font-bold mb-2">{asset.name}</h1>
                    <div className="flex items-center gap-4 text-muted">
                      <span className="flex items-center">
                        <MapPin className="h-4 w-4 mr-1" />
                        {asset.location}
                      </span>
                      <span className="px-3 py-1 bg-[rgba(var(--accent),0.20)] text-white rounded-full text-sm">
                        {asset.type}
                      </span>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm text-muted mb-1">ROI</div>
                    <div className="flex items-center text-green-500 text-2xl font-bold">
                      <TrendingUp className="h-5 w-5 mr-1" />
                      {asset.roi}%
                    </div>
                  </div>
                </div>

                <p className="text-muted leading-relaxed mb-8">{asset.description}</p>

                {/* Quick Stats */}
                <div className="grid grid-cols-3 gap-4 mb-8">
                  <div className="glass-card p-4 text-center">
                    <div className="text-2xl font-bold text-white mb-1">
                      {formatCurrency(asset.totalValue)}
                    </div>
                    <div className="text-sm text-muted">Total Value</div>
                  </div>
                  <div className="glass-card p-4 text-center">
                    <div className="text-2xl font-bold text-white mb-1">
                      {formatNumber(asset.totalTokens)}
                    </div>
                    <div className="text-sm text-muted">Total Tokens</div>
                  </div>
                  <div className="glass-card p-4 text-center">
                    <div className="text-2xl font-bold text-white mb-1">
                      {((asset.totalTokens - asset.availableTokens) / asset.totalTokens * 100).toFixed(0)}%
                    </div>
                    <div className="text-sm text-muted">Funded</div>
                  </div>
                </div>

                {/* Specifications */}
                <div>
                  <h3 className="font-display text-2xl font-bold mb-4">Specifications</h3>
                  <div className="grid md:grid-cols-2 gap-4">
                    {Object.entries(asset.specifications).map(([key, value]) => (
                      <div key={key} className="flex justify-between py-3 border-b border-[rgba(var(--accent),0.30)]">
                        <span className="text-muted">{key}</span>
                        <span className="font-semibold">{value}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </motion.div>
            </div>

            {/* Right Column - Purchase Widget */}
            <div className="lg:col-span-1">
              <motion.div
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.2 }}
                className="glass-card p-8 sticky top-32"
              >
                <h3 className="font-display text-2xl font-bold mb-6">Purchase Calculator</h3>

                <div className="space-y-6">
                  {/* Token Price */}
                  <div>
                    <div className="text-sm text-muted mb-1">Token Price</div>
                    <div className="text-3xl font-bold text-white">
                      {formatCurrency(asset.tokenPrice)}
                    </div>
                  </div>

                  {/* Available Tokens */}
                  <div>
                    <div className="text-sm text-muted mb-1">Available Tokens</div>
                    <div className="text-xl font-semibold">
                      {formatNumber(asset.availableTokens)} / {formatNumber(asset.totalTokens)}
                    </div>
                    <div className="w-full bg-white/10 rounded-full h-2 mt-2">
                      <div
                        className="bg-[rgb(var(--accent))] h-2 rounded-full"
                        style={{ width: `${((asset.totalTokens - asset.availableTokens) / asset.totalTokens) * 100}%` }}
                      />
                    </div>
                  </div>

                  {/* Token Amount Input */}
                  <div>
                    <label className="text-sm text-muted mb-2 block">Number of Tokens</label>
                    <input
                      type="number"
                      min="1"
                      max={asset.availableTokens}
                      value={tokenAmount}
                      onChange={(e) => setTokenAmount(Math.max(1, parseInt(e.target.value) || 1))}
                      className="w-full glass-card px-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))]"
                    />
                  </div>

                  {/* Purchase Summary */}
                  <div className="glass-card p-4 space-y-3">
                    <div className="flex justify-between">
                      <span className="text-muted">Total Price</span>
                      <span className="font-semibold">{formatCurrency(purchaseTotal)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted">Payment Method</span>
                      <span className="font-semibold text-white">Crypto</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted">Processing Fee</span>
                      <span className="font-semibold text-green-500">
                        {formatCurrency(purchaseTotal * 0.05)}
                      </span>
                    </div>
                  </div>

                  {/* Buy Button */}
                  <Button className="w-full py-6 text-lg" size="lg">
                    Buy Now with Crypto
                  </Button>

                  <p className="text-xs text-muted text-center">
                    By purchasing, you agree to our terms and conditions
                  </p>
                </div>
              </motion.div>
            </div>
          </div>
        </div>
      </main>
    </div>
  )
}



