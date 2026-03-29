import '../styles/globals.css'
import { Providers } from './providers'
import { OledBackground3D } from '@/components/background/OledBackground3D'

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
    <html lang="en" className="dark" suppressHydrationWarning>
      <body className="bg-background text-foreground antialiased">
        <OledBackground3D />
        <Providers>{children}</Providers>
      </body>
    </html>
  )
}