/** 日志类型定义 */

export interface LogEntry {
  timestamp: number;
  level: "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";
  target: string;
  message: string;
}

export type LogLevel = "all" | "DEBUG" | "INFO" | "WARN" | "ERROR";
