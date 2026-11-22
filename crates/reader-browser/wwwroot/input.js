export function isCompositionKey(event, composing = false) {
  return composing || event.isComposing || event.key === "Process" || event.key === "Dead" || event.keyCode === 229;
}

export function applyInputHints(input, mode, enter, capitalization, language = "") {
  // Plain TextBox has no transactional exact-edit correction protocol. Hints
  // never enable suggestions, replace the element, or touch value/selection.
  const attributes = {
    inputmode: mode, enterkeyhint: enter, autocapitalize: capitalization,
    autocorrect: "off", spellcheck: "false", writingsuggestions: "false",
    autocomplete: "off", lang: language || null,
  };
  for (const [name, value] of Object.entries(attributes)) {
    if (input.getAttribute(name) === value) continue;
    if (value === null) input.removeAttribute(name); else input.setAttribute(name, value);
  }
}

// Called synchronously by the native TopLevel's GotFocus event, even when
// two native fields share one DOM focus target. Exact-edit textarea is untouched.
export function inputHintsChanged(mode, enter, capitalization, language) {
  const input = document.getElementById("out")?.querySelector(":scope > .ferroui-input-element");
  if (input) applyInputHints(input, mode, enter, capitalization, language);
}

export function attachInputGuard(element) {
  let composing = false;
  const start = () => { composing = true; };
  const end = () => { composing = false; };
  const key = event => {
    // The framework's generic key callback has no composition flag. Leave
    // candidate-selection keys to the browser before app shortcuts see them.
    // Composition and beforeinput/input events still reach FerroUI unchanged.
    if (isCompositionKey(event, composing)) event.stopPropagation();
  };
  element.addEventListener("compositionstart", start, true);
  element.addEventListener("compositionend", end, true);
  element.addEventListener("focusout", end, true);
  element.addEventListener("keydown", key, true);
  element.addEventListener("keyup", key, true);
  return () => {
    element.removeEventListener("compositionstart", start, true);
    element.removeEventListener("compositionend", end, true);
    element.removeEventListener("focusout", end, true);
    element.removeEventListener("keydown", key, true);
    element.removeEventListener("keyup", key, true);
  };
}
