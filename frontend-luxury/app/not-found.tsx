import Link from 'next/link'
import { ArrowLeft, Home } from 'lucide-react'

export default function NotFound() {
  return (
    <section className="min-h-screen px-6 py-12">
      <div className="container mx-auto flex min-h-screen items-center">
        <div className="grid w-full items-center gap-12 lg:grid-cols-2">
          <div>
            <p className="text-sm font-medium text-white">404 error</p>
            <h1 className="mt-3 text-2xl font-semibold text-white md:text-3xl">
              Page not found
            </h1>
            <p className="mt-4 max-w-xl text-white/80">
              Sorry, the page you are looking for does not exist. Here are some helpful links.
            </p>

            <div className="mt-6 flex flex-col gap-3 sm:flex-row">
              <Link
                href="/"
                className="inline-flex w-full items-center justify-center gap-2 rounded-lg border border-[rgba(var(--accent),0.7)] bg-transparent px-5 py-2 text-sm text-white transition-colors duration-200 hover:bg-[rgba(var(--accent),0.10)] sm:w-auto"
              >
                <ArrowLeft className="h-5 w-5" />
                <span>Go Back</span>
              </Link>

              <Link
                href="/"
                className="inline-flex w-full items-center justify-center gap-2 rounded-lg bg-[rgb(var(--accent))] px-5 py-2 text-sm font-medium text-white transition-colors duration-200 hover:bg-[#4a7bff] sm:w-auto"
              >
                <Home className="h-5 w-5" />
                <span>Take Me Home</span>
              </Link>
            </div>
          </div>

          <div className="relative">
            <div className="glass-card mx-auto flex max-w-lg items-center justify-center p-10">
              <div className="flex h-64 w-full items-center justify-center rounded-2xl border border-[rgba(var(--accent),0.7)] bg-[linear-gradient(135deg,rgba(69,120,255,0.16),rgba(69,120,255,0.04))]">
                <div className="text-center">
                  <div className="text-7xl font-bold text-white">404</div>
                  <div className="mt-3 text-sm uppercase tracking-[0.28em] text-white/70">
                    Route Missing
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
