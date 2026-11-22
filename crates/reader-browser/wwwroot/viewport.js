// FerroUI owns one-finger scrolling. The browser still owns page magnification
// and the keyboard; its visual viewport can be smaller than the layout viewport.
export function viewportRectangle(layout, visual) {
  const positive = value => Number.isFinite(value) && value > 0;
  const offset = value => Number.isFinite(value) ? Math.max(0, value) : 0;
  // Reflowing to a pinched viewport would undo the user's magnification.
  if (visual && positive(visual.scale) && Math.abs(visual.scale - 1) > 0.01) return null;
  return {
    width: positive(visual?.width) ? visual.width : layout.width,
    height: positive(visual?.height) ? visual.height : layout.height,
    left: offset(visual?.offsetLeft),
    top: offset(visual?.offsetTop),
  };
}

export function attachViewport(runtime, element, window = globalThis) {
  let frame = 0;
  let previous;
  const pointer = window.matchMedia?.("(pointer: coarse)");
  const updatePointer = () => runtime.setTouchMode(Boolean(pointer?.matches));
  const update = () => {
    frame = 0;
    const root = window.document.documentElement;
    const rect = viewportRectangle({ width: root.clientWidth, height: window.innerHeight }, window.visualViewport);
    if (rect && (rect.width !== previous?.width || rect.height !== previous?.height ||
        rect.left !== previous?.left || rect.top !== previous?.top)) {
      for (const [name, value] of Object.entries(rect)) root.style.setProperty(`--reader-viewport-${name}`, `${value}px`);
      previous = rect;
    }
    // The host's measured box already excludes the CSS safe-area insets.
    const box = element.getBoundingClientRect();
    runtime.resizeViewport(box.width, box.height);
  };
  const schedule = () => { if (!frame) frame = window.requestAnimationFrame(update); };
  const observer = new window.ResizeObserver(schedule);
  observer.observe(element);
  window.addEventListener("resize", schedule);
  window.addEventListener("pageshow", schedule);
  window.visualViewport?.addEventListener("resize", schedule);
  window.visualViewport?.addEventListener("scroll", schedule);
  pointer?.addEventListener("change", updatePointer);
  updatePointer();
  update();
  return () => {
    observer.disconnect();
    if (frame) window.cancelAnimationFrame(frame);
    window.removeEventListener("resize", schedule);
    window.removeEventListener("pageshow", schedule);
    window.visualViewport?.removeEventListener("resize", schedule);
    window.visualViewport?.removeEventListener("scroll", schedule);
    pointer?.removeEventListener("change", updatePointer);
  };
}
