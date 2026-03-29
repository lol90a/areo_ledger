'use client'

import { Hero3D } from '@/components/hero/Hero3D'
import { Navbar } from '@/components/navbar/Navbar'
import { motion } from 'framer-motion'
import {
  ArrowRight,
  Blocks,
  Building2,
  CheckCircle2,
  Coins,
  Plane,
  Shield,
  Waves,
} from 'lucide-react'
import Link from 'next/link'
import { Button } from '@/components/ui/button'

const platformSignals = [
  { value: '$2.4B', label: 'Transaction volume represented across luxury inventory' },
  { value: '12.5K', label: 'Verified buyers and operators' },
  { value: '6', label: 'Supported crypto settlement rails' },
  { value: '45', label: 'Markets and jurisdictions supported' },
]

const capabilities = [
  {
    icon: Shield,
    title: 'Escrow visibility',
    description: 'Keep bookings, authorization, and state transitions visible through a controlled payment flow.',
  },
  {
    icon: Coins,
    title: 'Multi-chain settlement',
    description: 'Accept BTC, ETH, SOL, USDT, USDC, and BNB without fragmenting the user experience.',
  },
  {
    icon: Blocks,
    title: 'Operational workflow',
    description: 'Move from reservation to ownership updates inside one product surface.',
  },
]

const productAreas = [
  {
    icon: Plane,
    name: 'Private aviation',
    text: 'Flight reservations and aircraft inventory routed through the same booking and payment engine.',
  },
  {
    icon: Waves,
    name: 'Maritime assets',
    text: 'High-value yacht acquisitions with wallet-native payment rails and clear hold states.',
  },
  {
    icon: Building2,
    name: 'Real estate',
    text: 'Property discovery and purchase orchestration built for premium cross-border buyers.',
  },
]

const workflow = [
  {
    title: 'Create a hold on the asset',
    text: 'Reserve the inventory item before settlement, so buyers never lose context between discovery and payment.',
  },
  {
    title: 'Route the crypto payment',
    text: 'Let the buyer choose the chain while the platform maintains one coherent checkout and confirmation layer.',
  },
  {
    title: 'Close with ledger-backed proof',
    text: 'Reflect transaction completion, ownership updates, and audit-friendly state in the same system.',
  },
]

const sectionVariants = {
  hidden: {},
  show: {
    transition: {
      staggerChildren: 0.08,
      delayChildren: 0.05,
    },
  },
}

const fadeUp = {
  hidden: { opacity: 0, y: 26 },
  show: {
    opacity: 1,
    y: 0,
    transition: {
      duration: 0.7,
      ease: [0.22, 1, 0.36, 1],
    },
  },
}

export default function HomePage() {
  return (
    <main className="min-h-screen pb-16">
      <Navbar />
      <Hero3D />

      <section className="px-5 py-10 md:px-8 md:py-14">
        <motion.div
          initial="hidden"
          whileInView="show"
          viewport={{ once: true, amount: 0.25 }}
          variants={sectionVariants}
          className="section-shell grid gap-4 md:grid-cols-4"
        >
          {platformSignals.map((item, index) => (
            <motion.div
              key={item.label}
              variants={fadeUp}
              whileHover={{ y: -8, scale: 1.01 }}
              transition={{ duration: 0.25 }}
              className="glass-card relative overflow-hidden p-5"
            >
              <div className="absolute inset-x-0 top-0 h-px bg-[linear-gradient(90deg,transparent,rgba(69,120,255,0.9),transparent)]" />
              <div className="text-3xl font-semibold md:text-[2rem]">{item.value}</div>
              <p className="mt-2 text-sm leading-6 text-muted">{item.label}</p>
            </motion.div>
          ))}
        </motion.div>
      </section>

      <section className="px-5 py-16 md:px-8 md:py-24">
        <div className="section-shell grid gap-10 lg:grid-cols-[0.92fr_1.08fr] lg:items-start">
          <motion.div
            initial="hidden"
            whileInView="show"
            viewport={{ once: true, amount: 0.35 }}
            variants={sectionVariants}
            className="max-w-xl"
          >
            <motion.div variants={fadeUp} className="eyebrow">Why teams use AeroLedger</motion.div>
            <motion.h2 variants={fadeUp} className="mt-5 text-4xl leading-tight md:text-6xl">
              Infrastructure thinking packaged as a readable product experience.
            </motion.h2>
            <motion.p variants={fadeUp} className="mt-5 text-lg leading-8 text-muted">
              The structure stays the same, but the visual system is now cleaner: stronger typography, more spacing, flatter surfaces, and less decorative noise.
            </motion.p>
          </motion.div>

          <motion.div
            initial="hidden"
            whileInView="show"
            viewport={{ once: true, amount: 0.25 }}
            variants={sectionVariants}
            className="grid gap-4 md:grid-cols-3"
          >
            {capabilities.map((item) => (
              <motion.div
                key={item.title}
                variants={fadeUp}
                whileHover={{ y: -10, rotateX: 4, rotateY: -4 }}
                transition={{ duration: 0.25 }}
                className="interactive-card p-6"
                style={{ transformStyle: 'preserve-3d' }}
              >
                <item.icon className="h-6 w-6 text-white" />
                <h3 className="mt-5 text-2xl">{item.title}</h3>
                <p className="mt-3 text-sm leading-7 text-muted">{item.description}</p>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </section>

      <section className="px-5 py-16 md:px-8 md:py-24">
        <div className="section-shell grid gap-6 lg:grid-cols-[1.05fr_0.95fr]">
          <motion.div
            initial="hidden"
            whileInView="show"
            viewport={{ once: true, amount: 0.25 }}
            variants={sectionVariants}
            className="glass-card p-7 md:p-10"
          >
            <motion.div variants={fadeUp} className="eyebrow">Product scope</motion.div>
            <motion.h2 variants={fadeUp} className="mt-5 text-4xl md:text-5xl">
              Designed around luxury asset workflows, not generic crypto screens.
            </motion.h2>
            <motion.p variants={fadeUp} className="mt-5 max-w-2xl text-base leading-8 text-muted">
              AeroLedger serves reservation-heavy, high-trust categories. The homepage still speaks in those terms, but now with the cleaner rhythm and discipline of a professional SaaS product.
            </motion.p>
            <motion.div variants={sectionVariants} className="mt-8 grid gap-4 md:grid-cols-3">
              {productAreas.map((area) => (
                <motion.div
                  key={area.name}
                  variants={fadeUp}
                  whileHover={{ y: -8, backgroundColor: 'rgba(69,120,255,0.05)' }}
                  className="rounded-xl border border-[rgba(var(--frame-strong),0.7)] p-5"
                >
                  <area.icon className="h-6 w-6 text-white" />
                  <h3 className="mt-4 text-xl">{area.name}</h3>
                  <p className="mt-2 text-sm leading-7 text-muted">{area.text}</p>
                </motion.div>
              ))}
            </motion.div>
          </motion.div>

          <motion.div
            initial="hidden"
            whileInView="show"
            viewport={{ once: true, amount: 0.25 }}
            variants={sectionVariants}
            className="grid gap-4"
          >
            {workflow.map((step) => (
              <motion.div
                key={step.title}
                variants={fadeUp}
                whileHover={{ x: 10 }}
                className="interactive-card p-6"
              >
                <div className="flex items-center gap-3 text-sm font-medium text-white">
                  <CheckCircle2 className="h-4 w-4" />
                  {step.title}
                </div>
                <p className="mt-3 text-sm leading-7 text-muted">{step.text}</p>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </section>

      <section className="px-5 py-16 md:px-8 md:py-24">
        <div className="section-shell">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, amount: 0.3 }}
            transition={{ duration: 0.7, ease: [0.22, 1, 0.36, 1] }}
            whileHover={{ y: -6 }}
            className="glass-card relative overflow-hidden p-8 md:p-12"
          >
            <div className="absolute inset-y-0 right-0 w-1/3 bg-[radial-gradient(circle_at_center,rgba(69,120,255,0.14),transparent_65%)]" />
            <div className="relative flex flex-col gap-6 md:flex-row md:items-end md:justify-between">
              <div className="max-w-2xl">
                <div className="eyebrow">AeroLedger next step</div>
                <h2 className="mt-5 text-4xl md:text-5xl">
                  Bring the same visual system into marketplace, flights, and checkout.
                </h2>
                <p className="mt-4 text-base leading-8 text-muted">
                  The content is unchanged, but the presentation is now cleaner, more readable, and closer to the rhythm of an enterprise platform.
                </p>
              </div>
              <div className="flex flex-col gap-3 sm:flex-row">
                <Link href="/marketplace">
                  <Button size="lg" className="h-12 min-w-[180px]">
                    Open Marketplace
                    <ArrowRight className="ml-2 h-4 w-4" />
                  </Button>
                </Link>
                <Link href="/checkout">
                  <Button variant="outline" size="lg" className="h-12 min-w-[180px]">
                    Review Checkout
                  </Button>
                </Link>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      <footer className="px-5 pb-8 pt-4 text-center text-sm text-muted md:px-8">
        <div className="section-shell border-t border-[rgba(var(--frame-strong),0.7)] pt-6">
          AeroLedger enterprise interface for luxury asset settlement.
        </div>
      </footer>
    </main>
  )
}
