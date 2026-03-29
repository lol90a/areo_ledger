'use client'

import { useTheme } from '@/context/ThemeContext'
import { Sun, Moon } from 'lucide-react'
import { motion } from 'framer-motion'
import { cn } from '@/lib/utils'

export function ThemeToggle() {
  const { theme, toggleTheme } = useTheme()

  return (
    <motion.button
      whileHover={{ y: -2 }}
      whileTap={{ scale: 0.96 }}
      onClick={toggleTheme}
      className="relative flex h-11 w-20 items-center rounded-full border border-[rgb(var(--frame-strong))] bg-[rgb(var(--surface-float))]/85 px-1 shadow-[0_10px_30px_rgba(0,0,0,0.14)] backdrop-blur-xl transition-colors"
      aria-label="Toggle theme"
    >
      <motion.span
        layout
        transition={{ type: 'spring', stiffness: 380, damping: 28 }}
        className={cn(
          'absolute top-1 h-9 w-9 rounded-full bg-[linear-gradient(135deg,var(--gold-strong),var(--gold-soft))] shadow-[0_12px_30px_rgba(69,120,255,0.28)]',
          theme === 'dark' ? 'left-1' : 'left-10'
        )}
      />
      <span className="relative z-10 flex w-full items-center justify-between px-2 text-white">
        <Moon className="h-4 w-4" />
        <Sun className="h-4 w-4" />
      </span>
    </motion.button>
  )
}
