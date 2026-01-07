'use client';

import React from 'react';

export default function NavbarGate({
  children,
}: {
  children: React.ReactNode;
}) {
  // Tauri 模式下不需要认证检查，直接渲染子组件
  return <>{children}</>;
}
