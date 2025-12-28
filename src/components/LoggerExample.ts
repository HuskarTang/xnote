// 在组件中使用日志工具的示例

import logger from '@/utils/logger';

export function exampleFunction() {
  logger.info('Example function called');
  
  try {
    // 模拟一些操作
    const result = doSomething();
    logger.debug('Operation result:', result);
    return result;
  } catch (error) {
    logger.error('Error in example function:', error);
    throw error;
  }
}

function doSomething() {
  logger.debug('Doing something...');
  return { success: true, data: 'example' };
}