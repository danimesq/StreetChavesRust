import init, { StreetChavesWebRuntime } from "../pkg/street_chaves_rust.js";

const diagnostics = document.querySelector("#diagnostics");
const audioButton = document.querySelector("#enable-audio");

function report(error) {
  diagnostics.textContent = error instanceof Error ? error.message : String(error);
}

try {
  await init();

  const game = new StreetChavesWebRuntime("street-chaves");
  await game.load_assets("./assets/street-chaves-manifest.json");

  audioButton.addEventListener("click", async () => {
    try {
      await game.resume_audio();
      audioButton.disabled = true;
      audioButton.textContent = "Audio enabled";
    } catch (error) {
      report(error);
    }
  });

  const canvas = document.querySelector("#street-chaves");
  const resize = () => {
    const bounds = canvas.getBoundingClientRect();
    game.resize_canvas(
      Math.max(1, Math.round(bounds.width)),
      Math.max(1, Math.round(bounds.height)),
      window.devicePixelRatio || 1
    );
  };

  if ("ResizeObserver" in window) {
    new ResizeObserver(resize).observe(canvas);
  } else {
    window.addEventListener("resize", resize);
  }
  resize();

  function frame() {
    try {
      game.frame();
      const messages = game.take_diagnostics();
      if (messages.length > 0) {
        diagnostics.textContent = messages.join("\n");
      }
      if (!game.should_quit()) {
        requestAnimationFrame(frame);
      }
    } catch (error) {
      report(error);
    }
  }

  requestAnimationFrame(frame);
} catch (error) {
  report(error);
}
