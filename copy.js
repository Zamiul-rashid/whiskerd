// Copy buttons on command strips. Without clipboard access the button says how to copy by hand.
for (const strip of document.querySelectorAll("[data-copy]")) {
  const button = strip.querySelector(".copy");
  const code = strip.querySelector("code");
  if (!button || !code) continue;
  let timer;
  button.addEventListener("click", async () => {
    clearTimeout(timer);
    try {
      await navigator.clipboard.writeText(code.textContent.trim());
      button.dataset.state = "done";
      button.textContent = "Copied";
    } catch {
      const range = document.createRange();
      range.selectNodeContents(code);
      getSelection().removeAllRanges();
      getSelection().addRange(range);
      button.dataset.state = "fail";
      button.textContent = "Press Ctrl+C";
    }
    timer = setTimeout(() => {
      delete button.dataset.state;
      button.textContent = "Copy";
    }, 2000);
  });
}
