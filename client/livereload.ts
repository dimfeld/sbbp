export function startLiveReload() {
  window.startLiveReload = startLiveReload;
  const sse = new EventSource('/__livereload_status');
  sse.onerror = () => {
    pollForLive();
    sse.close();
  };

  let liveReloadActive = true;
  async function pollForLive() {
    if (!liveReloadActive) {
      return;
    }

    try {
      let res = await fetch('/api/healthz');
      if (res.ok) {
        window.location.reload();
      }
    } catch (e) {}

    if (liveReloadActive) {
      setTimeout(pollForLive, 250);
    }
  }

  window.stopLiveReload = () => {
    liveReloadActive = false;
    sse.close();
  };
}
