/** 应用配置 */
export interface AppConfig {
  /** 主题 dark/light */
  theme: "dark" | "light";
  /** 默认分组ID */
  defaultGroupId: number;
  /** 开机自启 */
  autoStart: boolean;
  /** 最小化到托盘 */
  minimizeToTray: boolean;
  /** 预警开关 */
  alertEnabled: boolean;
  /** 预警声音 */
  alertSound: boolean;
}

export const DEFAULT_CONFIG: AppConfig = {
  theme: "dark",
  defaultGroupId: 1,
  autoStart: false,
  minimizeToTray: true,
  alertEnabled: true,
  alertSound: true,
};
