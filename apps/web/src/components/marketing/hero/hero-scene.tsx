"use client";

import { useEffect, useRef, useState } from "react";
import * as THREE from "three";

export function HeroScene() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [hasWebGL, setHasWebGL] = useState<boolean>(true);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // Check prefers-reduced-motion
    const prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    // Check WebGL availability
    try {
      const canvas = document.createElement("canvas");
      const gl =
        canvas.getContext("webgl") || canvas.getContext("experimental-webgl");
      if (!gl) {
        setHasWebGL(false);
        return;
      }
    } catch {
      setHasWebGL(false);
      return;
    }

    const width = container.clientWidth || 600;
    const height = container.clientHeight || 500;

    // Scene, Camera, Renderer
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(
      45,
      width / height,
      0.1,
      1000
    );
    camera.position.z = 7;

    const renderer = new THREE.WebGLRenderer({
      alpha: true,
      antialias: true,
      powerPreference: "high-performance",
    });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    container.appendChild(renderer.domElement);

    // Group to hold the entire rescue core assembly
    const coreGroup = new THREE.Group();
    scene.add(coreGroup);

    // 1. Inner Core: Glowing crystalline icosahedron
    const innerGeo = new THREE.IcosahedronGeometry(1.2, 1);
    const innerMat = new THREE.MeshBasicMaterial({
      color: 0x72e5d0,
      wireframe: true,
      transparent: true,
      opacity: 0.35,
    });
    const innerCore = new THREE.Mesh(innerGeo, innerMat);
    coreGroup.add(innerCore);

    // 2. Dense Nucleus
    const nucleusGeo = new THREE.SphereGeometry(0.5, 16, 16);
    const nucleusMat = new THREE.MeshBasicMaterial({
      color: 0x72e5d0,
      wireframe: false,
      transparent: true,
      opacity: 0.85,
    });
    const nucleus = new THREE.Mesh(nucleusGeo, nucleusMat);
    coreGroup.add(nucleus);

    // 3. Middle Shield: Dodecahedron wireframe
    const midGeo = new THREE.DodecahedronGeometry(1.9, 0);
    const midMat = new THREE.MeshBasicMaterial({
      color: 0x57d6a0,
      wireframe: true,
      transparent: true,
      opacity: 0.25,
    });
    const midShield = new THREE.Mesh(midGeo, midMat);
    coreGroup.add(midShield);

    // 4. Outer Orbital Rings (representing risk thresholds & recovery horizons)
    const ringGeo1 = new THREE.TorusGeometry(2.5, 0.02, 16, 100);
    const ringMat1 = new THREE.MeshBasicMaterial({
      color: 0x72e5d0,
      transparent: true,
      opacity: 0.5,
    });
    const ring1 = new THREE.Mesh(ringGeo1, ringMat1);
    ring1.rotation.x = Math.PI / 3;
    coreGroup.add(ring1);

    const ringGeo2 = new THREE.TorusGeometry(2.8, 0.015, 16, 100);
    const ringMat2 = new THREE.MeshBasicMaterial({
      color: 0x57d6a0,
      transparent: true,
      opacity: 0.35,
    });
    const ring2 = new THREE.Mesh(ringGeo2, ringMat2);
    ring2.rotation.y = Math.PI / 4;
    ring2.rotation.x = -Math.PI / 6;
    coreGroup.add(ring2);

    const ringGeo3 = new THREE.TorusGeometry(3.1, 0.01, 16, 100);
    const ringMat3 = new THREE.MeshBasicMaterial({
      color: 0x1e2d3d,
      transparent: true,
      opacity: 0.6,
    });
    const ring3 = new THREE.Mesh(ringGeo3, ringMat3);
    ring3.rotation.z = Math.PI / 5;
    coreGroup.add(ring3);

    // 5. Ambient Particle Constellation (representing oracle observations)
    const particleCount = 200;
    const particlePositions = new Float32Array(particleCount * 3);
    for (let i = 0; i < particleCount * 3; i += 3) {
      const radius = 3.5 + Math.random() * 3.5;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(Math.random() * 2 - 1);
      particlePositions[i] = radius * Math.sin(phi) * Math.cos(theta);
      particlePositions[i + 1] = radius * Math.sin(phi) * Math.sin(theta);
      particlePositions[i + 2] = radius * Math.cos(phi);
    }
    const particleGeo = new THREE.BufferGeometry();
    particleGeo.setAttribute(
      "position",
      new THREE.BufferAttribute(particlePositions, 3)
    );
    const particleMat = new THREE.PointsMaterial({
      color: 0x72e5d0,
      size: 0.04,
      transparent: true,
      opacity: 0.6,
    });
    const particles = new THREE.Points(particleGeo, particleMat);
    scene.add(particles);

    // Mouse parallax tracking
    let targetMouseX = 0;
    let targetMouseY = 0;
    let currentMouseX = 0;
    let currentMouseY = 0;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      const x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      const y = -(((e.clientY - rect.top) / rect.height) * 2 - 1);
      targetMouseX = x * 0.4;
      targetMouseY = y * 0.4;
    };

    window.addEventListener("mousemove", handleMouseMove, { passive: true });

    // Responsive resize handler
    const handleResize = () => {
      if (!container) return;
      const newWidth = container.clientWidth;
      const newHeight = container.clientHeight;
      camera.aspect = newWidth / newHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(newWidth, newHeight);
    };

    const resizeObserver = new ResizeObserver(handleResize);
    resizeObserver.observe(container);

    // Animation Loop
    let animId: number;
    let clock = new THREE.Clock();

    const animate = () => {
      animId = requestAnimationFrame(animate);

      const elapsedTime = clock.getElapsedTime();
      const speedFactor = prefersReducedMotion ? 0.05 : 1.0;

      // Rotate components
      innerCore.rotation.x = elapsedTime * 0.25 * speedFactor;
      innerCore.rotation.y = elapsedTime * 0.35 * speedFactor;

      midShield.rotation.x = -elapsedTime * 0.15 * speedFactor;
      midShield.rotation.z = elapsedTime * 0.2 * speedFactor;

      ring1.rotation.z = elapsedTime * 0.1 * speedFactor;
      ring2.rotation.x = elapsedTime * 0.08 * speedFactor;
      ring3.rotation.y = -elapsedTime * 0.05 * speedFactor;

      particles.rotation.y = elapsedTime * 0.02 * speedFactor;

      // Core pulse
      const pulse = 1 + Math.sin(elapsedTime * 1.5) * 0.04;
      nucleus.scale.set(pulse, pulse, pulse);

      // Smooth mouse follow
      currentMouseX += (targetMouseX - currentMouseX) * 0.05;
      currentMouseY += (targetMouseY - currentMouseY) * 0.05;

      coreGroup.rotation.y = currentMouseX;
      coreGroup.rotation.x = -currentMouseY;

      renderer.render(scene, camera);
    };

    animate();

    // Clean up on unmount
    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener("mousemove", handleMouseMove);
      resizeObserver.disconnect();

      // Dispose Three.js objects to avoid memory leaks
      innerGeo.dispose();
      innerMat.dispose();
      nucleusGeo.dispose();
      nucleusMat.dispose();
      midGeo.dispose();
      midMat.dispose();
      ringGeo1.dispose();
      ringMat1.dispose();
      ringGeo2.dispose();
      ringMat2.dispose();
      ringGeo3.dispose();
      ringMat3.dispose();
      particleGeo.dispose();
      particleMat.dispose();

      renderer.dispose();
      if (renderer.domElement && container.contains(renderer.domElement)) {
        container.removeChild(renderer.domElement);
      }
    };
  }, []);

  if (!hasWebGL) {
    return (
      <div className="w-full h-full flex items-center justify-center">
        <div className="relative w-64 h-64 flex items-center justify-center">
          <div className="absolute inset-0 rounded-full border border-[var(--accent)]/20 animate-pulse" />
          <div className="absolute inset-4 rounded-full border border-[var(--risk-safe)]/30 animate-spin" />
          <div className="h-16 w-16 rounded-full bg-[var(--accent)]/10 border border-[var(--accent)]/40 flex items-center justify-center">
            <span className="text-[var(--accent)] font-mono text-2xl font-bold">◈</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="w-full h-full min-h-[420px] md:min-h-[520px] flex items-center justify-center pointer-events-auto cursor-grab active:cursor-grabbing"
      aria-label="3D Interactive Rescue Core Visualization"
      role="img"
    />
  );
}
