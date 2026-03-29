'use client'

import { motion } from 'framer-motion'
import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { ArrowRight, CheckCircle2 } from 'lucide-react'

const heroCopy = [
  'Reserve inventory before payment is finalized',
  'Route BTC, ETH, USDT, USDC, SOL, and BNB in one flow',
  'Keep audit-friendly status visible throughout checkout',
]

const metrics = [
  { label: 'Settlement rails', value: '6' },
  { label: 'Verified buyers', value: '12.5K' },
  { label: 'Regions served', value: '45' },
]

const containerVariants = {
  hidden: {},
  show: {
    transition: {
      staggerChildren: 0.12,
      delayChildren: 0.08,
    },
  },
}

const itemVariants = {
  hidden: { opacity: 0, y: 28 },
  show: {
    opacity: 1,
    y: 0,
    transition: {
      duration: 0.7,
      ease: [0.22, 1, 0.36, 1],
    },
  },
}

export function Hero3D() {
  return (
    <section className="relative overflow-hidden px-5 pb-16 pt-32 md:px-8 md:pb-24 md:pt-36">
      <div className="absolute inset-x-0 top-10 h-[28rem] bg-[radial-gradient(circle_at_center,rgba(69,120,255,0.18),transparent_62%)] blur-3xl" />

      <div className="section-shell relative">
        <motion.div
          initial="hidden"
          animate="show"
          variants={containerVariants}
          className="grid gap-12 lg:grid-cols-[1.15fr_0.85fr] lg:items-center"
        >
          <div className="max-w-3xl">
            <motion.div variants={itemVariants} className="eyebrow">
              Enterprise infrastructure for high-value asset settlement
            </motion.div>

            <motion.h1
              variants={itemVariants}
              className="mt-6 text-5xl leading-[0.95] md:text-7xl"
            >
              Buy and settle luxury assets through one clear operating layer.
            </motion.h1>

            <motion.p variants={itemVariants} className="mt-6 max-w-2xl text-lg leading-8 text-muted">
              AeroLedger connects booking, crypto payment, and ownership workflows in a product that feels structured, readable, and operationally trustworthy.
            </motion.p>

            <motion.div variants={itemVariants} className="mt-8 flex flex-col gap-4 sm:flex-row">
              <Link href="/marketplace">
                <Button size="lg" className="h-12 min-w-[180px]">
                  Explore Marketplace
                  <ArrowRight className="ml-2 h-4 w-4" />
                </Button>
              </Link>
              <Link href="/flights">
                <Button variant="outline" size="lg" className="h-12 min-w-[180px]">
                  View Flights
                </Button>
              </Link>
            </motion.div>

            <motion.div variants={itemVariants} className="mt-10 flex flex-wrap gap-3">
              {['Instant route clarity', 'Live settlement states', 'Crisp payment motion'].map((chip, index) => (
                <motion.div
                  key={chip}
                  animate={{ y: [0, -6, 0] }}
                  transition={{ duration: 4 + index * 0.4, repeat: Infinity, ease: 'easeInOut' }}
                  className="rounded-full border border-[rgba(var(--accent),0.55)] bg-[rgba(var(--accent),0.08)] px-4 py-2 text-sm text-white/90"
                >
                  {chip}
                </motion.div>
              ))}
            </motion.div>
          </div>

          <motion.div variants={itemVariants} className="relative">
            <motion.div
              animate={{ y: [0, -10, 0], rotate: [0, 1.2, 0] }}
              transition={{ duration: 8, repeat: Infinity, ease: 'easeInOut' }}
              className="glass-card relative overflow-hidden bg-[rgba(8,10,16,0.84)] p-6 backdrop-blur-sm md:p-8"
            >
              <div className="absolute inset-0 bg-[linear-gradient(135deg,rgba(69,120,255,0.14),transparent_34%,transparent_66%,rgba(69,120,255,0.08))]" />
              <div className="absolute -right-16 top-10 h-36 w-36 rounded-full bg-[rgba(69,120,255,0.14)] blur-3xl" />
              <div className="absolute -left-10 bottom-8 h-24 w-24 rounded-full bg-[rgba(109,216,255,0.12)] blur-3xl" />

              <div className="relative flex items-center justify-between border-b border-[rgba(var(--frame-strong),0.7)] pb-4">
                <div>
                  <div className="text-sm font-medium">AeroLedger Overview</div>
                  <div className="mt-1 text-sm text-muted">Reservation, payment, and ownership status</div>
                </div>
                <motion.span
                  animate={{ boxShadow: ['0 0 0 rgba(69,120,255,0)', '0 0 24px rgba(69,120,255,0.35)', '0 0 0 rgba(69,120,255,0)'] }}
                  transition={{ duration: 2.6, repeat: Infinity, ease: 'easeInOut' }}
                  className="status-chip"
                >
                  Live
                </motion.span>
              </div>

              <div className="relative mt-6 space-y-4">
                {heroCopy.map((item, index) => (
                  <motion.div
                    key={item}
                    initial={{ opacity: 0, x: 20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: 0.45 + index * 0.12, duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
                    whileHover={{ x: 8, borderColor: 'rgba(69,120,255,0.95)' }}
                    className="flex items-start gap-3 rounded-xl border border-[rgba(var(--frame-strong),0.7)] p-4"
                  >
                    <CheckCircle2 className="mt-0.5 h-4 w-4 text-white" />
                    <p className="text-sm leading-6 text-muted">{item}</p>
                  </motion.div>
                ))}
              </div>

              <motion.div variants={containerVariants} initial="hidden" animate="show" className="relative mt-6 grid gap-3 sm:grid-cols-3">
                {metrics.map((metric, index) => (
                  <motion.div
                    key={metric.label}
                    variants={itemVariants}
                    whileHover={{ y: -6, backgroundColor: 'rgba(69,120,255,0.08)' }}
                    transition={{ duration: 0.25 }}
                    className="rounded-xl border border-[rgba(var(--frame-strong),0.7)] bg-[rgba(255,255,255,0.01)] p-4"
                  >
                    <motion.div
                      animate={{ opacity: [0.8, 1, 0.8] }}
                      transition={{ duration: 2.2 + index * 0.3, repeat: Infinity, ease: 'easeInOut' }}
                      className="text-2xl font-semibold"
                    >
                      {metric.value}
                    </motion.div>
                    <div className="mt-1 text-sm text-muted">{metric.label}</div>
                  </motion.div>
                ))}
              </motion.div>
            </motion.div>
          </motion.div>
        </motion.div>
      </div>
    </section>
  )
}
