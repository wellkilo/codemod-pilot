import { getUserProfile } from './api';

async function loadProfile(id: string) {
  const user = await getUserProfile({ profileId: id });
  console.log(user.name);
  return user;
}

export function getUser(userId: string) {
  return getUserProfile({ profileId: userId });
}
