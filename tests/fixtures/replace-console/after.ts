function processData(data: any[]) {
  logger.info('Processing started');
  for (const item of data) {
    logger.info('Processing item:', item.id);
    try {
      transform(item);
    } catch (e) {
      logger.error('Failed to process:', e);
    }
  }
  logger.info('Processing complete');
}
