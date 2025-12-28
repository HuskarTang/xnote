/**
 * 日志工具类
 * 提供带文件名和行号的日志输出功能
 */

// 定义日志级别
export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3
}

// 日志配置接口
interface LoggerConfig {
  level: LogLevel;
  prefix?: string;
}

// 默认配置
const defaultConfig: LoggerConfig = {
  level: LogLevel.DEBUG,
  prefix: '[XNote]'
};

class Logger {
  private config: LoggerConfig;

  constructor(config: Partial<LoggerConfig> = {}) {
    this.config = { ...defaultConfig, ...config };
  }

  // 设置日志级别
  setLevel(level: LogLevel): void {
    this.config.level = level;
  }

  // 获取调用栈信息
  private getCallerInfo(): { fileName: string; lineNumber: number } {
    try {
      const stack = new Error().stack;
      if (stack) {
        // 查找调用栈中的文件信息
        const stackLines = stack.split('\n');
        // 跳过前几行（Error构造和getCallerInfo）
        for (let i = 3; i < stackLines.length; i++) {
          const line = stackLines[i];
          // 匹配文件名和行号
          const match = line.match(/(?:\(|at )([^)]*):(\d+):\d+\)?$/);
          if (match) {
            const fileName = match[1].split('/').pop() || match[1];
            const lineNumber = parseInt(match[2], 10);
            return { fileName, lineNumber };
          }
        }
      }
    } catch (e) {
      // 忽略错误
    }
    
    return { fileName: 'unknown', lineNumber: 0 };
  }

  // 格式化日志消息
  private formatMessage(level: string, message: any[]): string {
    const { fileName, lineNumber } = this.getCallerInfo();
    const timestamp = new Date().toISOString();
    const location = `${fileName}:${lineNumber}`;
    
    return `${this.config.prefix} ${timestamp} [${level}] [${location}] ${message.join(' ')}`;
  }

  // DEBUG 级别日志
  debug(...message: any[]): void {
    if (this.config.level <= LogLevel.DEBUG) {
      console.debug(this.formatMessage('DEBUG', message));
    }
  }

  // INFO 级别日志
  info(...message: any[]): void {
    if (this.config.level <= LogLevel.INFO) {
      console.info(this.formatMessage('INFO', message));
    }
  }

  // WARN 级别日志
  warn(...message: any[]): void {
    if (this.config.level <= LogLevel.WARN) {
      console.warn(this.formatMessage('WARN', message));
    }
  }

  // ERROR 级别日志
  error(...message: any[]): void {
    if (this.config.level <= LogLevel.ERROR) {
      console.error(this.formatMessage('ERROR', message));
    }
  }
}

// 创建默认实例
const logger = new Logger();

// 导出默认实例和类
export default logger;
export { Logger };