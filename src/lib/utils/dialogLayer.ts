// Dialogs open on top of one another in any order: About (F1, the tray menu)
// over any dialog, and the exit confirmation over any dialog, About included.
// The dialog that opened last has focus, so it must also be the one drawn on
// top: each dialog backdrop, when it appears, gets a higher z-index than every
// backdrop before it. Use as {@attach bringToFront} on the backdrop.
let topLayer = 100;

export function bringToFront(backdrop: HTMLElement) {
  topLayer += 1;
  backdrop.style.zIndex = String(topLayer);
}
