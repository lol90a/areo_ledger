'use client'

import { Hero3D } from '@/components/hero/Hero3D'
import { motion } from 'framer-motion'
import { Shield, TrendingUp, Zap, ArrowRight } from 'lucide-react'
import Link from 'next/link'
import { Button } from '@/components/ui/button'

export default function HomePage() {
  return (
    <main className="min-h-screen">
      {/* 3D Hero Section */}
      <Hero3D />

      {/* Features Section */}
      <section className="py-24 px-4">
        <div className="container mx-auto">
          <motion.div
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="font-display text-5xl font-bold mb-4">
              Why Choose <span className="text-gold">AeroLedger</span>
            </h2>
            <p className="text-xl text-muted max-w-2xl mx-auto">
              The premier platform for luxury asset purchases with cryptocurrency
            </p>
          </motion.div>

          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                icon: Shield,
                title: 'Secure Payments',
                description: 'All transactions secured with blockchain technology and verified wallets',
              },
              {
                icon: TrendingUp,
                title: 'Instant Ownership',
                description: 'Complete your purchase and receive ownership documents immediately',
              },
              {
                icon: Zap,
                title: '6 Cryptocurrencies',
                description: 'Pay with BTC, ETH, USDT, USDC, SOL, or BNB for maximum flexibility',
              },
            ].map((feature, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.2 }}
                whileHover={{ y: -10 }}
                className="glass-card p-8 text-center"
              >
                <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gold/20 mb-6">
                  <feature.icon className="h-8 w-8 text-gold" />
                </div>
                <h3 className="font-display text-2xl font-bold mb-4">{feature.title}</h3>
                <p className="text-muted">{feature.description}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Asset Categories */}
      <section className="py-24 px-4 bg-[#141414]">
        <div className="container mx-auto">
          <motion.div
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="font-display text-5xl font-bold mb-4">
              Luxury Asset <span className="text-gold">Categories</span>
            </h2>
          </motion.div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              { name: 'Private Jets', count: '24 Available', image: '✈️' },
              { name: 'Super Yachts', count: '18 Available', image: '🛥️' },
              { name: 'Real Estate', count: '42 Properties', image: '🏰' },
              { name: 'Luxury Cars', count: '36 Vehicles', image: '🏎️' },
            ].map((category, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, scale: 0.9 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                whileHover={{ scale: 1.05 }}
                className="glass-card p-8 text-center cursor-pointer"
              >
                <div className="text-6xl mb-4">{category.image}</div>
                <h3 className="font-display text-2xl font-bold mb-2">{category.name}</h3>
                <p className="text-gold">{category.count}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-24 px-4">
        <div className="container mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="glass-card p-12 text-center luxury-shadow"
          >
            <h2 className="font-display text-5xl font-bold mb-6">
              Ready to Start Purchasing?
            </h2>
            <p className="text-xl text-muted mb-8 max-w-2xl mx-auto">
              Join thousands of customers buying luxury assets with cryptocurrency
            </p>
            <Link href="/marketplace">
              <Button size="lg" className="text-lg px-12 py-6 h-auto">
                Browse Marketplace
                <ArrowRight className="ml-2 h-5 w-5" />
              </Button>
            </Link>
          </motion.div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-white/10 py-12 px-4">
        <div className="container mx-auto text-center text-muted">
          <p>&copy; 2025 AeroLedger. All rights reserved.</p>
        </div>
      </footer>
    </main>
  )
}
