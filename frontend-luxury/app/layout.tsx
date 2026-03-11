import '../styles/globals.css'
import { Inter, Playfair_Display } from 'next/font/google'
import { Providers } from './providers'

const inter = Inter({ 
  subsets: ['latin'],
  variable: '--font-inter',
  display: 'swap',
})

const playfair = Playfair_Display({ 
  subsets: ['latin'],
  variable: '--font-display',
  display: 'swap',
})

export const metadata = {
  title: 'AeroLedger - Buy Luxury Assets with Crypto',
  description: 'Buy private jets, yachts, real estate, and luxury cars with cryptocurrency',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" className={`${inter.variable} ${playfair.variable} dark`}>
      <body className="bg-[#0A0A0A] text-white antialiased">
        <Providers>{children}</Providers>
      </body>
    </html>
  )
}
