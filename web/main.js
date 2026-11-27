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
      await game.play_menu_music();
      audioButton.disabled = true;
      audioButton.textContent = "Audio enabled";
    } catch (error) {
      report(error);
    }
  });

  const canvas = document.querySelector("#street-chaves");
  const customCharacters = game.list_custom_characters();
  if (customCharacters.length > 0 && canvas.parentElement) {
    const roster = document.createElement("div");
    roster.setAttribute("aria-label", "Custom character roster");
    roster.style.cssText = "display:flex;gap:8px;align-items:center;justify-content:center;flex-wrap:wrap;padding:8px;font:14px system-ui";
    const label = document.createElement("label");
    label.textContent = "Custom fighter";
    const select = document.createElement("select");
    select.style.cssText = "max-width:220px;padding:7px;border-radius:8px";
    for (const character of customCharacters) {
      const option = document.createElement("option");
      option.value = character.id;
      option.textContent = character.name + " · " + character.id;
      select.append(option);
    }
    for (const [player, caption] of [[0, "Use for P1"], [1, "Use for P2"]]) {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = caption;
      button.style.cssText = "padding:7px 10px;border-radius:8px";
      button.addEventListener("click", () => {
        try {
          const runtimeId = game.select_custom_character(player, select.value);
          report("Selected " + select.value + " for P" + (player + 1) + " (runtime ID " + runtimeId + ").");
        } catch (error) {
          report(error);
        }
      });
      roster.append(button);
    }
    roster.prepend(label, select);
    canvas.parentElement.append(roster);
  }
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


  // Merge keyboard, gamepad and touch sources instead of letting one source release another.
  const gamepadButtons = Array.from({ length: 2 }, () => new Set());
  const touchButtons = Array.from({ length: 2 }, () => new Set());
  const lastPublishedButtons = Array.from({ length: 2 }, () => new Set());
  const gamepadDirections = [0, 0];
  const touchDirections = Array.from({ length: 2 }, () => ({ up: false, down: false, left: false, right: false }));

  function publishButton(player, slot) {
    const pressed = gamepadButtons[player].has(slot) || touchButtons[player].has(slot);
    const previous = lastPublishedButtons[player].has(slot);
    if (pressed === previous) return;
    if (pressed) lastPublishedButtons[player].add(slot);
    else lastPublishedButtons[player].delete(slot);
    game.set_virtual_button(player, slot, pressed);
  }

  function directionCode(dx, dy) {
    if (dx === 0 && dy === 0) return 0;
    if (dy < 0) return dx > 0 ? 2 : dx < 0 ? 8 : 1;
    if (dy > 0) return dx > 0 ? 4 : dx < 0 ? 6 : 5;
    return dx > 0 ? 3 : 7;
  }

  function publishDirection(player) {
    const touch = touchDirections[player];
    let dx = (touch.right ? 1 : 0) - (touch.left ? 1 : 0);
    let dy = (touch.down ? 1 : 0) - (touch.up ? 1 : 0);
    if (dx === 0 && dy === 0) {
      game.set_virtual_direction(player, gamepadDirections[player]);
    } else {
      game.set_virtual_direction(player, directionCode(Math.sign(dx), Math.sign(dy)));
    }
  }

  function pollGamepads() {
    if (!navigator.getGamepads) return;
    const pads = navigator.getGamepads();
    for (let player = 0; player < 2; player += 1) {
      const pad = pads[player];
      const next = new Set();
      let dx = 0;
      let dy = 0;
      if (pad) {
        const axisX = pad.axes?.[0] ?? 0;
        const axisY = pad.axes?.[1] ?? 0;
        if (axisX < -0.35) dx = -1;
        else if (axisX > 0.35) dx = 1;
        if (axisY < -0.35) dy = -1;
        else if (axisY > 0.35) dy = 1;
        if (pad.buttons?.[12]?.pressed) dy = -1;
        if (pad.buttons?.[13]?.pressed) dy = 1;
        if (pad.buttons?.[14]?.pressed) dx = -1;
        if (pad.buttons?.[15]?.pressed) dx = 1;
        for (let slot = 0; slot < 4; slot += 1) {
          if (pad.buttons?.[slot]?.pressed) next.add(slot);
        }
        if (pad.buttons?.[4]?.pressed) next.add(4);
        if (pad.buttons?.[5]?.pressed) next.add(5);
      }
      gamepadButtons[player].clear();
      for (const slot of next) gamepadButtons[player].add(slot);
      for (let slot = 0; slot <= 5; slot += 1) publishButton(player, slot);
      gamepadDirections[player] = directionCode(dx, dy);
      publishDirection(player);
    }
  }

  function addTouchControls() {
    if (!navigator.maxTouchPoints || !canvas.parentElement) return;
    const parent = canvas.parentElement;
    if (getComputedStyle(parent).position === "static") parent.style.position = "relative";
    const controls = document.createElement("div");
    controls.setAttribute("aria-label", "Touch controls");
    controls.style.cssText = "position:absolute;inset:auto 0 10px 0;display:flex;justify-content:space-between;align-items:end;padding:0 12px;z-index:5;pointer-events:none;user-select:none;touch-action:none";
    const style = document.createElement("style");
    style.textContent = ".street-touch-pad{display:grid;grid-template-columns:repeat(3,44px);grid-template-rows:repeat(3,44px);gap:3px;pointer-events:auto}.street-touch-actions{display:grid;grid-template-columns:repeat(2,52px);grid-template-rows:repeat(2,52px);gap:8px;pointer-events:auto}.street-touch-control{border:1px solid #dce5f3;border-radius:14px;background:#17263dcc;color:#fff;font:700 18px system-ui;touch-action:none;user-select:none}.street-touch-pad .street-touch-control[data-dir='up']{grid-area:1/2}.street-touch-pad .street-touch-control[data-dir='left']{grid-area:2/1}.street-touch-pad .street-touch-control[data-dir='right']{grid-area:2/3}.street-touch-pad .street-touch-control[data-dir='down']{grid-area:3/2}@media(pointer:fine){.street-touch-controls{display:none!important}}";
    document.head.append(style);

    const directions = document.createElement("div");
    directions.className = "street-touch-pad";
    for (const [name, glyph] of [["up", "▲"], ["left", "◀"], ["right", "▶"], ["down", "▼"]]) {
      const control = document.createElement("button");
      control.className = "street-touch-control";
      control.type = "button";
      control.dataset.dir = name;
      control.textContent = glyph;
      bindTouch(control, (down) => {
        touchDirections[0][name] = down;
        publishDirection(0);
      });
      directions.append(control);
    }

    const actions = document.createElement("div");
    actions.className = "street-touch-actions";
    for (const [slot, glyph] of [[0, "A"], [1, "B"], [2, "X"], [3, "Y"]]) {
      const control = document.createElement("button");
      control.className = "street-touch-control";
      control.type = "button";
      control.textContent = glyph;
      bindTouch(control, (down) => {
        if (down) touchButtons[0].add(slot);
        else touchButtons[0].delete(slot);
        publishButton(0, slot);
      });
      actions.append(control);
    }
    controls.append(directions, actions);
    parent.append(controls);
  }

  function bindTouch(element, onChange) {
    let activePointer = null;
    element.addEventListener("pointerdown", (event) => {
      event.preventDefault();
      activePointer = event.pointerId;
      element.setPointerCapture(activePointer);
      onChange(true);
    });
    const release = (event) => {
      if (activePointer !== event.pointerId) return;
      activePointer = null;
      onChange(false);
    };
    element.addEventListener("pointerup", release);
    element.addEventListener("pointercancel", release);
    element.addEventListener("lostpointercapture", () => {
      if (activePointer !== null) {
        activePointer = null;
        onChange(false);
      }
    });
  }

  addTouchControls();

  let wasHidden = document.hidden;
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      wasHidden = true;
    } else if (wasHidden) {
      for (let player = 0; player < 2; player += 1) {
        for (const slot of [...lastPublishedButtons[player]]) game.set_virtual_button(player, slot, false);
        lastPublishedButtons[player].clear();
        gamepadButtons[player].clear();
        touchButtons[player].clear();
        gamepadDirections[player] = 0;
        touchDirections[player] = { up: false, down: false, left: false, right: false };
        publishDirection(player);
      }
      game.reset_after_suspend();
      wasHidden = false;
    }
  });

  function frame() {
    try {
      pollGamepads();
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
