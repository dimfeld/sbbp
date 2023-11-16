export async function load({ url }) {
  const path = url.searchParams.get('path');
  if (!path) {
    return {
      file: null,
    };
  }

  return {
    file: null,
  };
}
