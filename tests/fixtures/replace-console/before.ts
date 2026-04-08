function processData(data: any[]) {
  console.log('Processing started');
  for (const item of data) {
    console.log('Processing item:', item.id);
    try {
      transform(item);
    } catch (e) {
      console.error('Failed to process:', e);
    }
  }
  console.log('Processing complete');
}
