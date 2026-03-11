/** @type {import('next').NextConfig} */
const nextConfig = {
  images: {
    domains: ['images.unsplash.com', 'localhost'],
  },
  transpilePackages: ['three'],
}

module.exports = nextConfig
