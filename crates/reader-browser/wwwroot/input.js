export function isCompositionKey(event, composing = false) {
  return composing || event.isComposing || event.key === "Process" || event.key === "Dead" || event.keyCode === 229;
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
