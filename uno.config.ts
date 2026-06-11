import {
  defineConfig,
  presetWind3,
  presetIcons,
} from 'unocss'

export default defineConfig({
  presets: [
    presetWind3(),
    presetIcons({
      scale: 1.2,
      cdn: 'https://esm.sh/',
    }),
  ],
  theme: {
    // 映射现有 CSS 变量到 UnoCSS theme
    colors: {
      bg: 'var(--color-bg)',
      'bg-secondary': 'var(--color-bg-secondary)',
      'bg-tertiary': 'var(--color-bg-tertiary)',
      surface: 'var(--color-surface)',
      'surface-hover': 'var(--color-surface-hover)',
      up: 'var(--color-up)',
      down: 'var(--color-down)',
      flat: 'var(--color-flat)',
      primary: 'var(--color-primary)',
      warning: 'var(--color-warning)',
      danger: 'var(--color-danger)',
      success: 'var(--color-success)',
      'text-primary': 'var(--color-text-primary)',
      'text-secondary': 'var(--color-text-secondary)',
      'text-tertiary': 'var(--color-text-tertiary)',
      border: 'var(--color-border)',
      'border-hover': 'var(--color-border-hover)',
    },
    boxShadow: {
      sm: 'var(--shadow-sm)',
      md: 'var(--shadow-md)',
      lg: 'var(--shadow-lg)',
      xl: 'var(--shadow-xl)',
      'glow-primary': 'var(--shadow-glow-primary)',
      'glow-up': 'var(--shadow-glow-up)',
      'glow-down': 'var(--shadow-glow-down)',
    },
    animation: {
      'shimmer': 'shimmer 1.5s infinite',
      'pulse-soft': 'pulse-soft 2s infinite',
      'fade-in': 'fade-in 0.3s ease-out',
      'slide-up': 'slide-up 0.3s ease-out',
      'slide-down': 'slide-down 0.3s ease-out',
    },
  },
  shortcuts: {
    // 卡片基础样式
    'card': 'bg-surface rounded-lg border border-border shadow-sm transition-all duration-200',
    'card-hover': 'hover:shadow-md hover:border-border-hover hover:-translate-y-0.5',
    'card-elevated': 'shadow-lg',

    // 涨跌色
    'text-up': 'text-up',
    'text-down': 'text-down',
    'text-flat': 'text-flat',
    'bg-up': 'bg-[var(--color-up-bg)]',
    'bg-down': 'bg-[var(--color-down-bg)]',

    // 按钮样式
    'btn': 'px-4 py-2 rounded-md transition-all duration-200 cursor-pointer',
    'btn-primary': 'bg-primary text-white hover:bg-primary/80',
    'btn-ghost': 'bg-transparent hover:bg-surface-hover',

    // 输入框样式
    'input-base': 'bg-surface border border-border rounded-md px-3 py-2 outline-none transition-all duration-200',
    'input-focus': 'focus:border-primary focus:shadow-glow-primary',

    // 动画
    'animate-out': 'transition-all duration-200 ease-out',
    'animate-spring': 'transition-all duration-300 [transition-timing-function:var(--ease-spring)]',
  },
  rules: [
    // 自定义规则
    ['glass', {
      'background': 'var(--glass-bg)',
      'backdrop-filter': 'var(--blur-md)',
    }],
    ['no-drag', { '-webkit-app-region': 'no-drag' }],
    ['drag', { '-webkit-app-region': 'drag' }],
  ],
})
