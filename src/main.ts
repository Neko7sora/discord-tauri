import "./styles.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("root element not found");
}

root.innerHTML = `
  <main class="redirect-shell" aria-live="polite">
    <section class="redirect-card">
      <img class="redirect-logo" src="/favicon.svg" alt="Discord logo" width="72" height="72" />
      <p class="redirect-label">Discord</p>
      <h1>discord.com/login を WebView で開いています</h1>
      <p class="redirect-copy">読み込みが終わるまで少し待ってください。</p>
    </section>
  </main>
`;
