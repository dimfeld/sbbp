import { logout } from 'filigree-web/auth/login.server';

export async function load(event) {
  await logout(event);
  return {};
}
