try {
  const root = document.documentElement;
  root.classList.toggle('dark', localStorage.getItem('maki-theme') !== 'light');
} catch {
  // Dark mode remains the default when storage is unavailable.
}

// Carry the live theme into the incoming document before Astro swaps it in.
document.addEventListener('astro:before-swap', (event) => {
  event.newDocument.documentElement.classList.toggle(
    'dark',
    document.documentElement.classList.contains('dark'),
  );
});
