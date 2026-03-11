import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'

export function middleware(request: NextRequest) {
  const path = request.nextUrl.pathname

  // Define protected routes
  const protectedRoutes = [
    '/dashboard',
    '/portfolio',
    '/transactions',
    '/cart',
    '/checkout',
    '/orders',
    '/profile',
    '/favorites'
  ]

  const adminRoutes = ['/admin']

  // Check if the path is protected
  const isProtectedRoute = protectedRoutes.some(route => path.startsWith(route))
  const isAdminRoute = adminRoutes.some(route => path.startsWith(route))

  // Get user from cookie or localStorage (we'll use a custom header)
  const userCookie = request.cookies.get('user')?.value
  
  if (isProtectedRoute || isAdminRoute) {
    // If no user cookie, redirect to signin
    if (!userCookie) {
      const url = new URL('/signin', request.url)
      url.searchParams.set('redirect', path)
      return NextResponse.redirect(url)
    }

    // Check admin access
    if (isAdminRoute) {
      try {
        const user = JSON.parse(userCookie)
        if (user.role !== 'admin') {
          return NextResponse.redirect(new URL('/dashboard', request.url))
        }
      } catch (error) {
        return NextResponse.redirect(new URL('/signin', request.url))
      }
    }
  }

  // Redirect to dashboard if already logged in and trying to access signin
  if (path === '/signin' && userCookie) {
    try {
      const user = JSON.parse(userCookie)
      const redirectUrl = user.role === 'admin' ? '/admin' : '/dashboard'
      return NextResponse.redirect(new URL(redirectUrl, request.url))
    } catch (error) {
      // Invalid cookie, continue to signin
    }
  }

  return NextResponse.next()
}

export const config = {
  matcher: [
    '/dashboard/:path*',
    '/portfolio/:path*',
    '/transactions/:path*',
    '/cart/:path*',
    '/checkout/:path*',
    '/orders/:path*',
    '/profile/:path*',
    '/favorites/:path*',
    '/admin/:path*',
    '/signin'
  ]
}
