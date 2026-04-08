import moment from 'moment';

const now = moment();
const formatted = moment(date).format('YYYY-MM-DD');
const tomorrow = moment().add(1, 'days');
const diff = moment(end).diff(moment(start), 'hours');
