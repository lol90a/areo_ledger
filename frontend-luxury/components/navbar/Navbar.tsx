'use client'

import Link from 'next/link'
import { motion, AnimatePresence } from 'framer-motion'
import { Button } from '@/components/ui/button'
import { Menu, X, ShoppingCart, User, Sparkles } from 'lucide-react'
import { useState, useEffect } from 'react'
import { useCart } from '@/context/CartContext'
import { useRouter, usePathname } from 'next/navigation'
import { session } from '@/lib/api'

export function Navbar() {
  const [isOpen, setIsOpen] = useState(false)
  const [isLoggedIn, setIsLoggedIn] = useState(false)
  const [userName, setUserName] = useState('')
  const { items } = useCart()
  const router = useRouter()
  const pathname = usePathname()

  useEffect(() => {
    const user = session.getUser()
    if (user) {
      setIsLoggedIn(true)
      setUserName(user.name || 'User')
    } else {
      setIsLoggedIn(false)
      setUserName('')
    }
  }, [])

  const handleLogout = () => {
    session.clear()
    setIsLoggedIn(false)
    setUserName('')
    router.push('/signin')
    setIsOpen(false)
  }

  const navItems = [
    { name: 'Marketplace', href: '/marketplace' },
    { name: 'Flights', href: '/flights' },
    { name: 'Calculator', href: '/calculator' },
    { name: 'Dashboard', href: '/dashboard' },
    { name: 'Portfolio', href: '/portfolio' },
  ]

  return (
    <nav className="fixed left-0 right-0 top-0 z-50 px-5 pt-4 md:px-8">
      <div className="section-shell">
        <motion.div
          initial={{ opacity: 0, y: -18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.65, ease: [0.22, 1, 0.36, 1] }}
          className="relative overflow-hidden rounded-[1.75rem] border border-[rgba(var(--accent),0.55)] bg-[rgba(8,10,16,0.78)] px-4 py-3 shadow-[0_18px_70px_rgba(0,0,0,0.36)] backdrop-blur-xl md:px-5"
        >
          <div className="absolute inset-0 bg-[linear-gradient(135deg,rgba(69,120,255,0.14),transparent_28%,transparent_68%,rgba(69,120,255,0.10))]" />
          <div className="absolute left-8 top-0 h-20 w-24 bg-[radial-gradient(circle,rgba(109,216,255,0.18),transparent_70%)] blur-2xl" />
          <div className="absolute right-6 top-1/2 h-16 w-20 -translate-y-1/2 bg-[radial-gradient(circle,rgba(69,120,255,0.16),transparent_72%)] blur-2xl" />

          <div className="relative flex items-center justify-between gap-4">
            <Link href="/" className="group flex min-w-0 items-center gap-3">
              <motion.div
                whileHover={{ scale: 1.05, rotate: -4 }}
                transition={{ duration: 0.25 }}
                className="flex h-11 w-11 items-center justify-center rounded-2xl border border-[rgba(var(--accent),0.65)] bg-[linear-gradient(145deg,rgba(69,120,255,1),rgba(109,216,255,0.7))] text-sm font-semibold text-white shadow-[0_10px_30px_rgba(69,120,255,0.35)]"
              >
                A
              </motion.div>
              <div className="min-w-0">
                <div className="truncate text-[1.02rem] font-semibold tracking-[-0.04em] text-white">AeroLedger</div>
                <div className="flex items-center gap-1 text-[10px] uppercase tracking-[0.24em] text-white/60">
                  <Sparkles className="h-3 w-3" />
                  Luxury Asset Settlement
                </div>
              </div>
            </Link>

            <div className="hidden items-center justify-center md:flex md:flex-1">
              <div className="flex items-center gap-1 rounded-full border border-[rgba(var(--accent),0.35)] bg-[rgba(255,255,255,0.03)] p-1.5">
                {navItems.map((item) => {
                  const isActive = pathname === item.href
                  return (
                    <Link
                      key={item.name}
                      href={item.href}
                      className={`relative rounded-full px-4 py-2 text-sm transition-all duration-300 ${
                        isActive
                          ? 'bg-[rgba(var(--accent),0.18)] text-white shadow-[inset_0_0_0_1px_rgba(69,120,255,0.55)]'
                          : 'text-white/72 hover:bg-[rgba(var(--accent),0.08)] hover:text-white'
                      }`}
                    >
                      {isActive && (
                        <motion.span
                          layoutId="nav-pill"
                          className="absolute inset-0 rounded-full border border-[rgba(var(--accent),0.55)]"
                          transition={{ type: 'spring', stiffness: 360, damping: 30 }}
                        />
                      )}
                      <span className="relative z-10">{item.name}</span>
                    </Link>
                  )
                })}
              </div>
            </div>

            <div className="hidden items-center gap-3 md:flex">
              <Link href="/cart" className="relative">
                <Button
                  variant="outline"
                  size="icon"
                  className="relative border-[rgba(var(--accent),0.55)] bg-[rgba(255,255,255,0.02)] text-white hover:bg-[rgba(var(--accent),0.10)]"
                >
                  <ShoppingCart className="h-4 w-4" />
                  {items.length > 0 && (
                    <span className="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-full bg-[rgb(var(--accent))] text-[10px] font-semibold text-white shadow-[0_8px_18px_rgba(69,120,255,0.35)]">
                      {items.length}
                    </span>
                  )}
                </Button>
              </Link>

              {isLoggedIn ? (
                <>
                  <div className="flex items-center gap-3 rounded-full border border-[rgba(var(--accent),0.4)] bg-[rgba(255,255,255,0.03)] px-4 py-2 text-sm text-white">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-[rgba(var(--accent),0.16)]">
                      <User className="h-4 w-4 text-white" />
                    </div>
                    <div>
                      <div className="text-[11px] uppercase tracking-[0.18em] text-white/45">Member</div>
                      <div className="max-w-[120px] truncate font-medium text-white">{userName}</div>
                    </div>
                  </div>
                  <Button variant="ghost" className="text-white/80 hover:text-white" onClick={handleLogout}>
                    Logout
                  </Button>
                </>
              ) : (
                <>
                  <Link href="/signin">
                    <Button variant="ghost" className="text-white/80 hover:text-white">Sign In</Button>
                  </Link>
                  <Link href="/marketplace">
                    <Button className="shadow-[0_12px_32px_rgba(69,120,255,0.32)]">Get Started</Button>
                  </Link>
                </>
              )}
            </div>

            <button
              className="inline-flex h-11 w-11 items-center justify-center rounded-2xl border border-[rgba(var(--accent),0.55)] bg-[rgba(255,255,255,0.03)] text-white md:hidden"
              onClick={() => setIsOpen(!isOpen)}
              aria-label="Toggle navigation"
            >
              {isOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
            </button>
          </div>
        </motion.div>

        <AnimatePresence>
          {isOpen && (
            <motion.div
              initial={{ opacity: 0, y: -14 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
              className="relative mt-3 overflow-hidden rounded-[1.6rem] border border-[rgba(var(--accent),0.55)] bg-[rgba(8,10,16,0.88)] p-4 shadow-[0_18px_60px_rgba(0,0,0,0.34)] backdrop-blur-xl md:hidden"
            >
              <div className="absolute inset-0 bg-[linear-gradient(180deg,rgba(69,120,255,0.10),transparent_35%,transparent_65%,rgba(69,120,255,0.08))]" />

              <div className="relative space-y-2">
                {navItems.map((item, index) => {
                  const isActive = pathname === item.href
                  return (
                    <motion.div
                      key={item.name}
                      initial={{ opacity: 0, x: -10 }}
                      animate={{ opacity: 1, x: 0 }}
                      transition={{ delay: index * 0.04 }}
                    >
                      <Link
                        href={item.href}
                        className={`block rounded-2xl px-4 py-3 text-sm transition-all ${
                          isActive
                            ? 'border border-[rgba(var(--accent),0.55)] bg-[rgba(var(--accent),0.12)] text-white'
                            : 'text-white/76 hover:bg-[rgba(var(--accent),0.08)] hover:text-white'
                        }`}
                        onClick={() => setIsOpen(false)}
                      >
                        {item.name}
                      </Link>
                    </motion.div>
                  )
                })}
              </div>

              <div className="relative mt-4 flex flex-col gap-2 border-t border-[rgba(var(--accent),0.28)] pt-4">
                {isLoggedIn ? (
                  <>
                    <div className="flex items-center gap-3 rounded-2xl border border-[rgba(var(--accent),0.38)] bg-[rgba(255,255,255,0.03)] px-4 py-3 text-sm text-white">
                      <div className="flex h-9 w-9 items-center justify-center rounded-full bg-[rgba(var(--accent),0.16)]">
                        <User className="h-4 w-4 text-white" />
                      </div>
                      <div>
                        <div className="text-[11px] uppercase tracking-[0.18em] text-white/45">Member</div>
                        <div className="font-medium text-white">{userName}</div>
                      </div>
                    </div>
                    <Button variant="ghost" className="w-full text-white/80 hover:text-white" onClick={handleLogout}>
                      Logout
                    </Button>
                  </>
                ) : (
                  <>
                    <Link href="/signin" onClick={() => setIsOpen(false)}>
                      <Button variant="ghost" className="w-full text-white/80 hover:text-white">Sign In</Button>
                    </Link>
                    <Link href="/marketplace" onClick={() => setIsOpen(false)}>
                      <Button className="w-full shadow-[0_12px_32px_rgba(69,120,255,0.32)]">Get Started</Button>
                    </Link>
                  </>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </nav>
  )
}
