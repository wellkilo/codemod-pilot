import { fetchUserInfo } from './api';

async function loadProfile(id: string) {
  const user = await fetchUserInfo({ userId: id });
  console.log(user.name);
  return user;
}

export function getUser(userId: string) {
  return fetchUserInfo({ userId });
}
