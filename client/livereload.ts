import htmx from 'htmx.org';

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
      if (!res.ok) {
        throw new Error('server still down');
      }

      await htmx.ajax('GET', window.location.href, { target: 'body' }).catch((e) => {
        console.error('AJAX reload failed', e);
        window.location.reload();
      });

      setTimeout(startLiveReload);
    } catch (e) {
      if (liveReloadActive) {
        setTimeout(pollForLive, 250);
      }
    }
  }

  window.stopLiveReload = () => {
    liveReloadActive = false;
    sse.close();
  };
}
