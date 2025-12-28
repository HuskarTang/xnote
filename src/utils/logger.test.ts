// 测试日志功能
import logger from './logger';

// 测试不同级别的日志
logger.debug('This is a debug message');
logger.info('This is an info message');
logger.warn('This is a warning message');
logger.error('This is an error message');

// 测试带参数的日志
const user = { name: 'John', age: 30 };
logger.info('User data:', user);

// 测试在不同文件中的日志
export function testFunction() {
  logger.debug('Inside testFunction');
  return 'test';
}