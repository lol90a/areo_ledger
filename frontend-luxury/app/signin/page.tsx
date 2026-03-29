'use client'

import { useState } from 'react'
import { motion } from 'framer-motion'
import { Button } from '@/components/ui/button'
import { Mail, Lock, User } from 'lucide-react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { api } from '@/lib/api'

export default function SignInPage() {
  const router = useRouter()
  const [isAdmin, setIsAdmin] = useState(false)
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [name, setName] = useState('')
  const [isSignUp, setIsSignUp] = useState(false)
  const [submitting, setSubmitting] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setSubmitting(true)

    try {
      const response = isSignUp
        ? await api.auth.signup({ name, email, password })
        : await api.auth.login({ email, password })

      const user = response.data
      if (isAdmin && user.role !== 'admin') {
        alert('This account does not have admin access.')
        setSubmitting(false)
        return
      }

      if (isSignUp) {
        alert(`Account created successfully! Welcome ${user.name}!`)
        setIsSignUp(false)
      }

      router.push(user.role === 'admin' ? '/admin' : '/dashboard')
    } catch (error: any) {
      console.error(isSignUp ? 'Signup error:' : 'Login error:', error)
      const message = error?.response?.data?.error || error?.response?.data?.message || 'Please check if the backend is running and try again.'
      alert(`${isSignUp ? 'Signup' : 'Login'} failed: ${message}`)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center px-4 py-20">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="w-full max-w-md"
      >
        <div className="text-center mb-8">
          <Link href="/" className="inline-block">
            <h1 className="font-display text-4xl font-bold mb-2">
              Aero<span className="text-white">Ledger</span>
            </h1>
          </Link>
          <p className="text-muted">
            {isSignUp ? 'Create your account' : 'Welcome back'}
          </p>
        </div>

        <div className="glass-card p-8">
          <div className="flex gap-2 mb-6">
            <button
              onClick={() => setIsAdmin(false)}
              className={`flex-1 py-2 rounded-lg transition-all ${
                !isAdmin ? 'bg-[rgb(var(--accent))] text-white font-semibold' : 'glass-card'
              }`}
            >
              User
            </button>
            <button
              onClick={() => setIsAdmin(true)}
              className={`flex-1 py-2 rounded-lg transition-all ${
                isAdmin ? 'bg-[rgb(var(--accent))] text-white font-semibold' : 'glass-card'
              }`}
            >
              Admin
            </button>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            {isSignUp && (
              <div>
                <label className="block text-sm text-muted mb-2">Name</label>
                <div className="relative">
                  <User className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted" />
                  <input
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="John Doe"
                    required
                    className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))]"
                  />
                </div>
              </div>
            )}

            <div>
              <label className="block text-sm text-muted mb-2">Email</label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted" />
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder={isAdmin ? 'admin@aeroledger.com' : 'you@example.com'}
                  required
                  className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))]"
                />
              </div>
            </div>

            <div>
              <label className="block text-sm text-muted mb-2">Password</label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted" />
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="********"
                  required
                  className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))]"
                />
              </div>
            </div>

            <Button type="submit" className="w-full py-6 text-lg" disabled={submitting}>
              {submitting ? 'Please wait...' : isSignUp ? 'Sign Up' : 'Sign In'}
            </Button>
          </form>

          <div className="mt-6 text-center">
            <button
              onClick={() => setIsSignUp(!isSignUp)}
              className="text-white hover:underline"
            >
              {isSignUp ? 'Already have an account? Sign In' : "Don't have an account? Sign Up"}
            </button>
          </div>
        </div>
      </motion.div>
    </div>
  )
}


