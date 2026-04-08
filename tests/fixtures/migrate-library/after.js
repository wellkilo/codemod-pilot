import dayjs from 'dayjs';

const now = dayjs();
const formatted = dayjs(date).format('YYYY-MM-DD');
const tomorrow = dayjs().add(1, 'day');
const diff = dayjs(end).diff(dayjs(start), 'hour');
