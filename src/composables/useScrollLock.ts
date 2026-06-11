/**
 * 防止滚动穿透
 * 当弹窗/抽屉打开时，禁止背景滚动
 */
import { watch, onUnmounted } from "vue";

export function useScrollLock(isLocked: () => boolean) {
  let originalOverflow = "";

  function lock() {
    originalOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
  }

  function unlock() {
    document.body.style.overflow = originalOverflow;
  }

  watch(isLocked, (locked) => {
    if (locked) {
      lock();
    } else {
      unlock();
    }
  });

  onUnmounted(() => {
    unlock();
  });

  return { lock, unlock };
}
