import init, { StreetChavesWebRuntime } from "./pkg/street_chaves.js";

const diagnostics = document.querySelector("#diagnostics");
const audioButton = document.querySelector("#enable-audio");

function report(error) {
  diagnostics.textContent = error instanceof Error ? error.message : String(error);
}

try {
  await init();

  const game = new StreetChavesWebRuntime("street-chaves");
  const preferencesKey = "street-chaves-arcade-preferences-v1";
  try {
    const savedPreferences = window.localStorage.getItem(preferencesKey);
    if (savedPreferences) {
      game.load_preferences(savedPreferences);
    }
  } catch (error) {
    report(`Could not restore arcade preferences: ${error}`);
  }
  await game.load_assets("./web-export.json");

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

  let wasHidden = document.hidden;
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      wasHidden = true;
    } else if (wasHidden) {
      game.reset_after_suspend();
      wasHidden = false;
    }
  });

  function frame() {
    try {
      game.frame();
      const updatedPreferences = game.take_preferences_json();
      if (updatedPreferences) {
        try {
          window.localStorage.setItem(preferencesKey, updatedPreferences);
        } catch (error) {
          report(`Could not save arcade preferences: ${error}`);
        }
      }
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
