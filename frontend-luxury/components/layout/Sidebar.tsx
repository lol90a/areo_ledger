'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { cn } from '@/lib/utils'
import {
  LayoutDashboard,
  Briefcase,
  ArrowLeftRight,
  BarChart3,
  Wallet,
  Settings,
  LogOut,
} from 'lucide-react'

const menuItems = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Portfolio', href: '/portfolio', icon: Briefcase },
  { name: 'Transactions', href: '/transactions', icon: ArrowLeftRight },
  { name: 'Analytics', href: '/analytics', icon: BarChart3 },
  { name: 'Wallet', href: '/wallet', icon: Wallet },
  { name: 'Settings', href: '/settings', icon: Settings },
]

export function Sidebar() {
  const pathname = usePathname()

  return (
    <aside className="flex w-64 flex-col border-r border-[rgba(var(--frame-strong),0.7)] bg-[rgb(var(--surface-float))]">
      <div className="border-b border-[rgba(var(--frame-strong),0.7)] p-6">
        <Link href="/" className="flex items-center space-x-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-[rgb(var(--accent))] text-sm font-semibold text-white">
            A
          </div>
          <div>
            <div className="text-lg font-semibold tracking-[-0.03em]">AeroLedger</div>
            <div className="text-[10px] uppercase tracking-[0.22em] text-muted">Workspace</div>
          </div>
        </Link>
      </div>

      <nav className="flex-1 space-y-2 p-4">
        {menuItems.map((item) => {
          const Icon = item.icon
          const isActive = pathname === item.href

          return (
            <Link
              key={item.name}
              href={item.href}
              className={cn(
                'flex items-center space-x-3 rounded-xl px-4 py-3 text-sm transition-all',
                isActive
                  ? 'border border-[rgba(var(--accent),0.35)] bg-[rgba(var(--accent),0.08)] text-white'
                  : 'text-muted hover:bg-[rgb(var(--surface-highlight))] hover:text-white'
              )}
            >
              <Icon className={cn('h-5 w-5', isActive ? 'text-white' : '')} />
              <span>{item.name}</span>
            </Link>
          )
        })}
      </nav>

      <div className="border-t border-[rgba(var(--frame-strong),0.7)] p-4">
        <div className="flex items-center space-x-3 rounded-xl border border-[rgba(var(--frame-strong),0.7)] px-4 py-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[rgba(var(--accent),0.1)] text-sm font-semibold text-white">
            JD
          </div>
          <div className="flex-1">
            <div className="text-sm font-medium">John Doe</div>
            <div className="text-xs text-muted">Investor</div>
          </div>
          <button className="text-muted transition-colors hover:text-white">
            <LogOut className="h-5 w-5" />
          </button>
        </div>
      </div>
    </aside>
  )
}
