'use client'

import { Canvas } from '@react-three/fiber'
import { OrbitControls, Environment, PerspectiveCamera } from '@react-three/drei'
import { Suspense } from 'react'
import { motion } from 'framer-motion'
import { Button } from '@/components/ui/button'
import Link from 'next/link'
import { ArrowRight } from 'lucide-react'

function Jet() {
  return (
    <mesh rotation={[0, Math.PI / 4, 0]}>
      {/* Simple jet representation - replace with GLTF model */}
      <group>
        {/* Fuselage */}
        <mesh position={[0, 0, 0]}>
          <cylinderGeometry args={[0.3, 0.3, 3, 32]} />
          <meshStandardMaterial color="#D4AF37" metalness={0.9} roughness={0.1} />
        </mesh>
        {/* Wings */}
        <mesh position={[0, 0, 0]} rotation={[0, 0, Math.PI / 2]}>
          <boxGeometry args={[4, 0.1, 0.8]} />
          <meshStandardMaterial color="#D4AF37" metalness={0.9} roughness={0.1} />
        </mesh>
        {/* Tail */}
        <mesh position={[0, 0.5, -1.3]} rotation={[0, 0, 0]}>
          <boxGeometry args={[0.1, 1, 0.6]} />
          <meshStandardMaterial color="#D4AF37" metalness={0.9} roughness={0.1} />
        </mesh>
      </group>
    </mesh>
  )
}

function Scene() {
  return (
    <>
      <PerspectiveCamera makeDefault position={[0, 2, 8]} />
      <ambientLight intensity={0.5} />
      <spotLight position={[10, 10, 10]} angle={0.15} penumbra={1} intensity={1} />
      <pointLight position={[-10, -10, -10]} intensity={0.5} />
      <Suspense fallback={null}>
        <Jet />
        <Environment preset="sunset" />
      </Suspense>
      <OrbitControls
        enableZoom={false}
        autoRotate
        autoRotateSpeed={0.5}
        maxPolarAngle={Math.PI / 2}
        minPolarAngle={Math.PI / 3}
      />
    </>
  )
}

export function Hero3D() {
  return (
    <section className="relative h-screen flex items-center justify-center overflow-hidden">
      {/* 3D Canvas Background */}
      <div className="absolute inset-0">
        <Canvas>
          <Scene />
        </Canvas>
      </div>

      {/* Gradient Overlay */}
      <div className="absolute inset-0 bg-gradient-to-b from-transparent via-[#0A0A0A]/50 to-[#0A0A0A]" />

      {/* Content */}
      <div className="relative z-10 container mx-auto px-4 text-center">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          <h1 className="font-display text-6xl md:text-8xl font-bold mb-6 leading-tight">
            Purchase Luxury Assets with{' '}
            <span className="text-gradient-gold">Cryptocurrency</span>
          </h1>
          
          <p className="text-xl md:text-2xl text-muted max-w-3xl mx-auto mb-12">
            Book flights, buy private jets, luxury yachts, premium real estate, and exotic cars with BTC, ETH, USDT, USDC, SOL, BNB
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link href="/marketplace">
              <Button size="lg" className="text-lg px-8 py-6 h-auto">
                Browse Marketplace
                <ArrowRight className="ml-2 h-5 w-5" />
              </Button>
            </Link>
            <Link href="/flights">
              <Button size="lg" variant="outline" className="text-lg px-8 py-6 h-auto">
                Book Flights
              </Button>
            </Link>
          </div>
        </motion.div>

        {/* Floating Stats Cards */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.3 }}
          className="mt-20 grid grid-cols-2 md:grid-cols-4 gap-4 md:gap-8"
        >
          {[
            { label: 'Total Sales', value: '$2.4B' },
            { label: 'Customers', value: '12,500+' },
            { label: 'Assets Sold', value: '850+' },
            { label: 'Countries', value: '45+' },
          ].map((stat, index) => (
            <motion.div
              key={index}
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.5 + index * 0.1 }}
              className="glass-card p-4 md:p-6 luxury-shadow"
            >
              <div className="text-2xl md:text-4xl font-display font-bold text-gold mb-2">
                {stat.value}
              </div>
              <div className="text-xs md:text-sm text-muted">{stat.label}</div>
            </motion.div>
          ))}
        </motion.div>
      </div>
    </section>
  )
}
