export function formatDuration(seconds: number) {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor(seconds / 60) % 60;
  const secs = seconds % 60;

  let segments = [hours, minutes, secs];
  if (!segments[0]) {
    segments = segments.slice(1);
  }

  return segments.map((s) => s.toString().padStart(2, '0')).join(':');
}
