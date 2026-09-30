import { useData } from 'vitepress'

// PLAN-715 T-02：主题态统一走 VitePress 运行时状态。
// useData().isDark 即 VueUse useDark（storageKey=vitepress-theme-appearance，
// 与站点既有存储 key 一致，appearance:'dark' 的初始偏好由 VitePress 处理）。
// 旧实现自建 ref+独立读写与默认主题双源，会在切页/跨组件时漂移，已移除。
export function useDarkMode() {
  const { isDark } = useData()
  return {
    isDark,
    toggle: () => {
      isDark.value = !isDark.value
      return isDark.value
    },
  }
}
