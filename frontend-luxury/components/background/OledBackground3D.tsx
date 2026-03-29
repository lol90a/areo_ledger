'use client'

import { Canvas, useFrame } from '@react-three/fiber'
import { PerspectiveCamera } from '@react-three/drei'
import { Suspense, useEffect, useRef, useState } from 'react'
import * as THREE from 'three'

function BackgroundOrbitals() {
  const group = useRef<THREE.Group>(null)
  const ringA = useRef<THREE.Mesh>(null)
  const ringB = useRef<THREE.Mesh>(null)
  const ringC = useRef<THREE.Mesh>(null)
  const core = useRef<THREE.Mesh>(null)

  useFrame((state) => {
    const t = state.clock.elapsedTime
    if (group.current) {
      group.current.rotation.y = t * 0.05
      group.current.position.y = Math.sin(t * 0.2) * 0.15
    }
    if (ringA.current) ringA.current.rotation.z = t * 0.12
    if (ringB.current) {
      ringB.current.rotation.x = 1.1 + Math.sin(t * 0.18) * 0.08
      ringB.current.rotation.y = t * 0.09
    }
    if (ringC.current) {
      ringC.current.rotation.x = 0.4
      ringC.current.rotation.y = t * -0.07
    }
    if (core.current) {
      const scale = 1 + Math.sin(t * 1.1) * 0.04
      core.current.scale.setScalar(scale)
      core.current.rotation.y = t * 0.18
    }
  })

  return (
    <group ref={group} position={[1.8, 0.2, -1.5]}>
      <mesh ref={ringA}>
        <torusGeometry args={[4.3, 0.015, 10, 120]} />
        <meshBasicMaterial color="#4a78ff" transparent opacity={0.22} />
      </mesh>
      <mesh ref={ringB}>
        <torusGeometry args={[5.8, 0.018, 10, 140]} />
        <meshBasicMaterial color="#6dd8ff" transparent opacity={0.12} />
      </mesh>
      <mesh ref={ringC}>
        <torusGeometry args={[7.2, 0.012, 10, 160]} />
        <meshBasicMaterial color="#8ca7ff" transparent opacity={0.1} />
      </mesh>
      <mesh ref={core}>
        <icosahedronGeometry args={[0.85, 1]} />
        <meshBasicMaterial color="#5d86ff" transparent opacity={0.16} wireframe />
      </mesh>
      <mesh position={[2.8, 1.4, 0.8]}>
        <sphereGeometry args={[0.06, 12, 12]} />
        <meshBasicMaterial color="#6dd8ff" transparent opacity={0.8} />
      </mesh>
      <mesh position={[-3.4, -1.2, -0.5]}>
        <sphereGeometry args={[0.08, 12, 12]} />
        <meshBasicMaterial color="#4a78ff" transparent opacity={0.7} />
      </mesh>
      <mesh position={[1.5, -2.4, 1.1]}>
        <sphereGeometry args={[0.05, 12, 12]} />
        <meshBasicMaterial color="#ffffff" transparent opacity={0.45} />
      </mesh>
    </group>
  )
}

function Scene() {
  return (
    <>
      <PerspectiveCamera makeDefault position={[0, 0, 11]} fov={36} />
      <color attach="background" args={['#020304']} />
      <fog attach="fog" args={['#020304', 8, 18]} />
      <ambientLight intensity={0.18} />
      <pointLight position={[1, 2, 6]} intensity={2.8} color="#4a78ff" />
      <pointLight position={[-4, -1, 4]} intensity={1.8} color="#6dd8ff" />
      <Suspense fallback={null}>
        <BackgroundOrbitals />
      </Suspense>
    </>
  )
}

export function OledBackground3D() {
  const [showCanvas, setShowCanvas] = useState(false)

  useEffect(() => {
    const media = window.matchMedia('(prefers-reduced-motion: reduce)')
    if (media.matches) return
    const id = window.setTimeout(() => setShowCanvas(true), 120)
    return () => window.clearTimeout(id)
  }, [])

  return (
    <div className="pointer-events-none fixed inset-0 -z-10 overflow-hidden">
      <div className="absolute inset-0 bg-black" />
      <div className="absolute inset-0 animate-[pulse_12s_ease-in-out_infinite] bg-[radial-gradient(circle_at_top_right,rgba(74,120,255,0.09),transparent_26%),radial-gradient(circle_at_left_top,rgba(109,216,255,0.05),transparent_18%)]" />
      <div className="absolute left-[8%] top-[18%] h-48 w-48 animate-[float_16s_ease-in-out_infinite] rounded-full bg-[rgba(69,120,255,0.08)] blur-3xl" />
      <div className="absolute bottom-[12%] right-[10%] h-64 w-64 animate-[float_19s_ease-in-out_infinite] rounded-full bg-[rgba(109,216,255,0.06)] blur-3xl" />
      {showCanvas ? (
        <Canvas dpr={[1, 1.25]} gl={{ antialias: false, alpha: true, powerPreference: 'high-performance' }}>
          <Scene />
        </Canvas>
      ) : null}
      <div className="absolute inset-0 bg-[linear-gradient(180deg,rgba(2,3,4,0.14),rgba(2,3,4,0.5)_58%,rgba(2,3,4,0.82))]" />
    </div>
  )
}
