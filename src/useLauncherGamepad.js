import { onBeforeUnmount, onMounted, watch } from "vue";

import { playHover } from "./sfx";

const DIRECTION_DEADZONE = 0.55;
const INITIAL_REPEAT_DELAY = 220;
const REPEAT_INTERVAL = 110;
const SCROLL_STEP = 140;
const FOCUS_SELECTOR = [
  "button:not([disabled])",
  "input:not([disabled]):not(.sr-only):not([data-controller-skip='true'])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "[data-controller-clickable='true']",
].join(", ");

function getEnabledValue(source) {
  if (typeof source === "function") return Boolean(source());
  if (source && typeof source === "object" && "value" in source) {
    return Boolean(source.value);
  }
  return Boolean(source);
}

function removeFocusClasses(element) {
  if (!element) return;
  element.classList.remove(
    "controller-nav-focused",
    "controller-nav-focused-big",
    "controller-nav-focused-small"
  );
}

function isElementVisible(element) {
  if (!(element instanceof HTMLElement)) return false;
  if (element.matches(":disabled")) return false;
  if (element.getAttribute("aria-hidden") === "true") return false;
  const style = window.getComputedStyle(element);
  if (
    style.display === "none" ||
    style.visibility === "hidden" ||
    style.pointerEvents === "none"
  ) {
    return false;
  }
  const rect = element.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
}

function focusClassForElement(element) {
  const explicit = String(element?.dataset?.controllerSize ?? "").trim().toLowerCase();
  if (explicit === "small") return "controller-nav-focused-small";
  if (explicit === "big") return "controller-nav-focused-big";

  const rect = element.getBoundingClientRect();
  return rect.width <= 80 && rect.height <= 80
    ? "controller-nav-focused-small"
    : "controller-nav-focused-big";
}

function priorityForElement(element) {
  const raw = Number(element?.dataset?.controllerPriority ?? Number.POSITIVE_INFINITY);
  return Number.isFinite(raw) ? raw : Number.POSITIVE_INFINITY;
}

function centerForRect(rect) {
  return {
    x: rect.left + rect.width / 2,
    y: rect.top + rect.height / 2,
  };
}

function scoreDirectionalCandidate(currentRect, candidateRect, direction) {
  const current = centerForRect(currentRect);
  const candidate = centerForRect(candidateRect);
  const dx = candidate.x - current.x;
  const dy = candidate.y - current.y;

  let primary = 0;
  let secondary = 0;
  if (direction === "up") {
    if (dy >= -2) return Number.POSITIVE_INFINITY;
    primary = Math.abs(dy);
    secondary = Math.abs(dx);
  } else if (direction === "down") {
    if (dy <= 2) return Number.POSITIVE_INFINITY;
    primary = Math.abs(dy);
    secondary = Math.abs(dx);
  } else if (direction === "left") {
    if (dx >= -2) return Number.POSITIVE_INFINITY;
    primary = Math.abs(dx);
    secondary = Math.abs(dy);
  } else {
    if (dx <= 2) return Number.POSITIVE_INFINITY;
    primary = Math.abs(dx);
    secondary = Math.abs(dy);
  }

  return primary * 1000 + secondary;
}

function getScrollContainer(element, scope) {
  let current = element instanceof HTMLElement ? element : null;
  while (current && current !== scope) {
    if (current.scrollHeight > current.clientHeight + 2) {
      return current;
    }
    current = current.parentElement;
  }

  if (scope instanceof HTMLElement && scope.scrollHeight > scope.clientHeight + 2) {
    return scope;
  }

  return scope?.querySelector?.(".scrollbar") ?? null;
}

function tryAdjustFocusedElement(element, direction) {
  if (!(element instanceof HTMLElement)) return false;

  const toggleState = String(element.dataset?.controllerToggleState ?? "").trim();
  if (toggleState === "on" || toggleState === "off") {
    const shouldToggleOn = direction === "right" && toggleState === "off";
    const shouldToggleOff = direction === "left" && toggleState === "on";
    if (shouldToggleOn || shouldToggleOff) {
      element.click?.();
      return true;
    }
    if (direction === "left" || direction === "right") {
      return true;
    }
  }

  if (element instanceof HTMLInputElement && element.type === "range") {
    const currentValue = Number(element.value ?? 0);
    const min = Number(element.min ?? 0);
    const max = Number(element.max ?? 100);
    const step = Number(element.step ?? 1) || 1;
    const nextValue =
      direction === "left"
        ? Math.max(min, currentValue - step)
        : Math.min(max, currentValue + step);
    if (nextValue === currentValue) return true;
    element.value = String(nextValue);
    element.dispatchEvent(new Event("input", { bubbles: true }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
    return true;
  }

  return false;
}

function getNodeId(element) {
  const raw = String(element?.dataset?.controllerNode ?? "").trim();
  return raw || null;
}

function getDirectionalNodeId(element, direction) {
  if (!(element instanceof HTMLElement)) return null;
  const key = `controller${direction.charAt(0).toUpperCase()}${direction.slice(1)}`;
  const raw = String(element.dataset?.[key] ?? "").trim();
  return raw || null;
}

function isTextEntryElement(element) {
  if (element instanceof HTMLInputElement) {
    const inputType = String(element.type ?? "").toLowerCase();
    return ![
      "range",
      "checkbox",
      "radio",
      "button",
      "submit",
      "reset",
      "file",
      "color",
    ].includes(inputType);
  }
  return (
    element instanceof HTMLTextAreaElement ||
    element instanceof HTMLSelectElement
  );
}

function suppressInputFocusSfx(element) {
  if (!isTextEntryElement(element)) return;
  element.dataset.controllerSuppressFocusSfx = "true";
  window.requestAnimationFrame(() => {
    if (element?.dataset?.controllerSuppressFocusSfx === "true") {
      delete element.dataset.controllerSuppressFocusSfx;
    }
  });
}

function usesManualControllerFocus(element) {
  return (
    isTextEntryElement(element) &&
    String(element?.dataset?.controllerFocusMode ?? "").trim().toLowerCase() === "manual"
  );
}

function emitControllerNavFocus(element) {
  if (!(element instanceof HTMLElement)) return;
  element.dispatchEvent(
    new CustomEvent("controller-nav-focus", {
      bubbles: true,
    })
  );
}

export function useLauncherGamepad(options) {
  let frameHandle = 0;
  let focusedElement = null;
  let controllerEngaged = false;
  let lastDirection = null;
  let directionStartedAt = 0;
  let lastDirectionRepeatAt = 0;
  let previousButtons = [];
  let graphState = {};
  let graphId = null;
  const lastFocusedNodeByGraphId = new Map();
  const lastFocusedElementByGraphId = new Map();

  function getScope() {
    return typeof options.resolveScope === "function"
      ? options.resolveScope()
      : options.resolveScope?.value ?? options.resolveScope ?? null;
  }

  function getGraph() {
    const source =
      typeof options.resolveGraph === "function"
        ? options.resolveGraph()
        : options.resolveGraph?.value ?? options.resolveGraph ?? null;
    return source && typeof source === "object" ? source : null;
  }

  function graphKeyFor(graph) {
    if (!graph) return null;
    return String(graph.id ?? "__anonymous_graph__");
  }

  function resetGraphState(graph) {
    graphState = typeof graph?.getInitialState === "function" ? graph.getInitialState() : {};
    graphId = graphKeyFor(graph);
  }

  function ensureGraphState() {
    const graph = getGraph();
    const nextGraphId = graphKeyFor(graph);
    if (nextGraphId !== graphId) {
      resetGraphState(graph);
    }
    return graph;
  }

  function clearFocusedElement() {
    const currentGraph = getGraph();
    const currentGraphId = graphKeyFor(currentGraph) ?? graphId;
    if (currentGraphId && focusedElement instanceof HTMLElement) {
      lastFocusedElementByGraphId.set(currentGraphId, focusedElement);
      const currentNodeId = getNodeId(focusedElement);
      if (currentNodeId) {
        lastFocusedNodeByGraphId.set(currentGraphId, currentNodeId);
      }
    }
    removeFocusClasses(focusedElement);
    focusedElement = null;
  }

  function refreshFocusedElementClass() {
    removeFocusClasses(focusedElement);
    if (!controllerEngaged || !(focusedElement instanceof HTMLElement)) {
      return;
    }
    focusedElement.classList.add("controller-nav-focused", focusClassForElement(focusedElement));
    if (!usesManualControllerFocus(focusedElement) && typeof focusedElement.focus === "function") {
      suppressInputFocusSfx(focusedElement);
      focusedElement.focus({ preventScroll: true });
    }
  }

  function applyFocus(element, { playSound = false } = {}) {
    if (!(element instanceof HTMLElement)) return false;
    if (focusedElement === element) {
      if (controllerEngaged) {
        refreshFocusedElementClass();
      }
      return true;
    }
    if (controllerEngaged) {
      const scope = getScope();
      const activeElement = document.activeElement;
      if (
        activeElement instanceof HTMLElement &&
        activeElement !== element &&
        isTextEntryElement(activeElement) &&
        (!(scope instanceof HTMLElement) || scope.contains(activeElement))
      ) {
        activeElement.blur();
      }
    }
    removeFocusClasses(focusedElement);
    focusedElement = element;
    const currentGraphId = graphKeyFor(getGraph());
    if (currentGraphId) {
      lastFocusedElementByGraphId.set(currentGraphId, focusedElement);
      const currentNodeId = getNodeId(focusedElement);
      if (currentNodeId) {
        lastFocusedNodeByGraphId.set(currentGraphId, currentNodeId);
      }
    }
    if (controllerEngaged) {
      focusedElement.classList.add("controller-nav-focused", focusClassForElement(focusedElement));
    }
    if (playSound && controllerEngaged) {
      playHover();
    }
    if (controllerEngaged) {
      emitControllerNavFocus(focusedElement);
    }
    if (
      controllerEngaged &&
      !usesManualControllerFocus(focusedElement) &&
      typeof focusedElement.focus === "function"
    ) {
      suppressInputFocusSfx(focusedElement);
      focusedElement.focus({ preventScroll: true });
    }
    return true;
  }

  function getFocusableElements() {
    const scope = getScope();
    if (!(scope instanceof HTMLElement)) return [];

    const elements = Array.from(scope.querySelectorAll(FOCUS_SELECTOR)).filter(isElementVisible);
    return Array.from(new Set(elements)).sort((left, right) => {
      const priorityDelta = priorityForElement(left) - priorityForElement(right);
      if (priorityDelta !== 0) return priorityDelta;
      const leftRect = left.getBoundingClientRect();
      const rightRect = right.getBoundingClientRect();
      const topDelta = leftRect.top - rightRect.top;
      if (Math.abs(topDelta) > 8) return topDelta;
      return leftRect.left - rightRect.left;
    });
  }

  function getGraphNodeElements() {
    const scope = getScope();
    if (!(scope instanceof HTMLElement)) return [];

    return Array.from(scope.querySelectorAll("[data-controller-node]")).filter(isElementVisible);
  }

  function getNodeElement(nodeId) {
    if (!nodeId) return null;
    return (
      getGraphNodeElements().find((element) => getNodeId(element) === nodeId) ?? null
    );
  }

  function isControllerFocusableElement(element) {
    const scope = getScope();
    return (
      element instanceof HTMLElement &&
      element.isConnected &&
      (!(scope instanceof HTMLElement) || scope.contains(element)) &&
      isElementVisible(element) &&
      (element.matches(FOCUS_SELECTOR) || Boolean(getNodeId(element)))
    );
  }

  function getVisibleNodeIds(prefix = "") {
    return getGraphNodeElements()
      .map((element) => getNodeId(element))
      .filter(Boolean)
      .filter((nodeId) => (prefix ? nodeId.startsWith(prefix) : true));
  }

  function sortNodeIdsByLeft(nodeIds) {
    return [...nodeIds].sort((leftId, rightId) => {
      const leftRect = getNodeElement(leftId)?.getBoundingClientRect();
      const rightRect = getNodeElement(rightId)?.getBoundingClientRect();
      if (!leftRect && !rightRect) return 0;
      if (!leftRect) return 1;
      if (!rightRect) return -1;
      const leftCenter = centerForRect(leftRect);
      const rightCenter = centerForRect(rightRect);
      const topDelta = leftCenter.y - rightCenter.y;
      if (Math.abs(topDelta) > 8) return topDelta;
      return leftCenter.x - rightCenter.x;
    });
  }

  function findNearestNodeId(nodeIds, fromNodeId) {
    const fromElement = getNodeElement(fromNodeId);
    if (!fromElement) return nodeIds[0] ?? null;

    const fromCenter = centerForRect(fromElement.getBoundingClientRect());
    let bestId = null;
    let bestScore = Number.POSITIVE_INFINITY;

    for (const nodeId of nodeIds) {
      const element = getNodeElement(nodeId);
      if (!element) continue;
      const center = centerForRect(element.getBoundingClientRect());
      const dx = center.x - fromCenter.x;
      const dy = center.y - fromCenter.y;
      const score = dx * dx + dy * dy;
      if (score < bestScore) {
        bestScore = score;
        bestId = nodeId;
      }
    }

    return bestId;
  }

  function activateElement(element) {
    if (!(element instanceof HTMLElement)) return false;

    if (isTextEntryElement(element)) {
      element.focus();
      return true;
    }

    if (typeof element.click === "function") {
      element.click();
      return true;
    }

    return false;
  }

  function buildGraphHelpers() {
    return {
      scope: getScope(),
      state: graphState,
      currentNodeId: getNodeId(focusedElement),
      focusedElement,
      getNodeId,
      getNodeElement,
      getFocusableElements,
      getVisibleNodeIds,
      sortNodeIdsByLeft,
      findNearestNodeId,
      focusElement(element, opts = {}) {
        if (!(element instanceof HTMLElement)) return false;
        return applyFocus(element, opts);
      },
      adjustFocused(direction) {
        return tryAdjustFocusedElement(focusedElement, direction);
      },
      focusNode(nodeId, opts = {}) {
        const element = getNodeElement(nodeId);
        if (!element) return false;
        return applyFocus(element, opts);
      },
      activateNode(nodeId) {
        const element = getNodeElement(nodeId);
        if (!element) return false;
        return activateElement(element);
      },
    };
  }

  function resolveInitialNode(graph) {
    if (!graph) return null;
    return typeof graph.initialNode === "function"
      ? graph.initialNode(buildGraphHelpers())
      : graph.initialNode ?? null;
  }

  function restoreGraphFocus(graph, { playSound = false } = {}) {
    if (!graph) return false;

    const activeElement = document.activeElement;
    if (isControllerFocusableElement(activeElement)) {
      return applyFocus(activeElement, { playSound });
    }

    const currentGraphId = graphKeyFor(graph);
    const rememberedElement =
      currentGraphId ? lastFocusedElementByGraphId.get(currentGraphId) : null;
    if (isControllerFocusableElement(rememberedElement)) {
      return applyFocus(rememberedElement, { playSound });
    }

    const rememberedNodeId =
      currentGraphId ? lastFocusedNodeByGraphId.get(currentGraphId) : null;
    if (rememberedNodeId && applyFocus(getNodeElement(rememberedNodeId), { playSound })) {
      return true;
    }

    const initialNodeId = resolveInitialNode(graph);
    return initialNodeId ? applyFocus(getNodeElement(initialNodeId), { playSound }) : false;
  }

  function focusFirstAvailable() {
    const graph = ensureGraphState();
    if (graph && restoreGraphFocus(graph)) {
      return;
    }

    const scope = getScope();
    const activeElement = document.activeElement;
    if (
      scope instanceof HTMLElement &&
      activeElement instanceof HTMLElement &&
      scope.contains(activeElement) &&
      isElementVisible(activeElement) &&
      activeElement.matches(FOCUS_SELECTOR)
    ) {
      if (applyFocus(activeElement)) {
        return;
      }
    }

    const elements = getFocusableElements();
    if (!elements.length) {
      clearFocusedElement();
      return;
    }
    applyFocus(elements[0]);
  }

  function moveFocus(direction) {
    const graph = ensureGraphState();
    if (graph) {
      if (!(focusedElement instanceof HTMLElement) || !isElementVisible(focusedElement)) {
        if (
          restoreGraphFocus(graph) &&
          focusedElement instanceof HTMLElement &&
          isElementVisible(focusedElement)
        ) {
          moveFocus(direction);
          return;
        }
        focusFirstAvailable();
        return;
      }

      const currentNodeId = getNodeId(focusedElement);

      const result = typeof graph.move === "function"
        ? graph.move({
            direction,
            currentNodeId,
            ...buildGraphHelpers(),
          })
        : null;

      if (typeof result === "string") {
        buildGraphHelpers().focusNode(result, { playSound: true });
      } else if (result === true) {
        return;
      } else if (result && typeof result === "object" && result.focusElement instanceof HTMLElement) {
        applyFocus(result.focusElement, {
          playSound: result.playSound !== false,
        });
      } else if (result && typeof result === "object" && result.focus) {
        buildGraphHelpers().focusNode(result.focus, {
          playSound: result.playSound !== false,
        });
      }
      return;
    }

    const elements = getFocusableElements();
    if (!elements.length) {
      clearFocusedElement();
      return;
    }

    if (!focusedElement || !elements.includes(focusedElement)) {
      applyFocus(elements[0]);
      return;
    }

    if (
      (direction === "left" || direction === "right") &&
      tryAdjustFocusedElement(focusedElement, direction)
    ) {
      return;
    }

    const explicitTargetNodeId = getDirectionalNodeId(focusedElement, direction);
    if (explicitTargetNodeId) {
      const explicitTarget = getNodeElement(explicitTargetNodeId);
      if (explicitTarget && isElementVisible(explicitTarget)) {
        applyFocus(explicitTarget, { playSound: true });
        return;
      }
    }

    const currentRect = focusedElement.getBoundingClientRect();
    let bestElement = null;
    let bestScore = Number.POSITIVE_INFINITY;

    for (const candidate of elements) {
      if (candidate === focusedElement) continue;
      const score = scoreDirectionalCandidate(
        currentRect,
        candidate.getBoundingClientRect(),
        direction
      );
      if (score < bestScore) {
        bestScore = score;
        bestElement = candidate;
      }
    }

    if (bestElement) {
      applyFocus(bestElement, { playSound: true });
    }
  }

  function activateFocusedElement() {
    const graph = ensureGraphState();
    if (graph) {
      if (!(focusedElement instanceof HTMLElement) || !isElementVisible(focusedElement)) {
        if (
          restoreGraphFocus(graph) &&
          focusedElement instanceof HTMLElement &&
          isElementVisible(focusedElement)
        ) {
          activateFocusedElement();
          return;
        }
        focusFirstAvailable();
        return;
      }

      const currentNodeId = getNodeId(focusedElement);
      const handled = typeof graph.activate === "function"
        ? graph.activate({
            currentNodeId,
            ...buildGraphHelpers(),
          })
        : false;
      if (handled) return;
    }

    const elements = getFocusableElements();
    if (!elements.length) {
      clearFocusedElement();
      return;
    }

    if (!focusedElement || !elements.includes(focusedElement)) {
      applyFocus(elements[0]);
    }

    activateElement(focusedElement);
  }

  function handleBack() {
    const scope = getScope();
    if (!(scope instanceof HTMLElement)) return;

    const activeInput = document.activeElement;
    if (
      activeInput instanceof HTMLElement &&
      scope.contains(activeInput) &&
      isTextEntryElement(activeInput)
    ) {
      activeInput.blur();
      return;
    }

    const graph = ensureGraphState();
    if (graph && typeof graph.back === "function") {
      if (!(focusedElement instanceof HTMLElement) || !isElementVisible(focusedElement)) {
        if (
          restoreGraphFocus(graph) &&
          focusedElement instanceof HTMLElement &&
          isElementVisible(focusedElement)
        ) {
          handleBack();
          return;
        }
      }
      const handled = graph.back({
        currentNodeId: getNodeId(focusedElement),
        ...buildGraphHelpers(),
      });
      if (handled) return;
    }

    if (typeof options.onBack === "function") {
      options.onBack();
    }
  }

  function scrollScope(direction) {
    const scope = getScope();
    if (!(scope instanceof HTMLElement)) return;
    const container = getScrollContainer(focusedElement, scope);
    if (!(container instanceof HTMLElement)) return;
    container.scrollBy({
      top: direction * (options.scrollStep ?? SCROLL_STEP),
      behavior: "smooth",
    });
  }

  function readDirection(gamepad) {
    if (!gamepad) return null;

    if (gamepad.buttons[12]?.pressed) return "up";
    if (gamepad.buttons[13]?.pressed) return "down";
    if (gamepad.buttons[14]?.pressed) return "left";
    if (gamepad.buttons[15]?.pressed) return "right";

    const axisX = Number(gamepad.axes?.[0] ?? 0);
    const axisY = Number(gamepad.axes?.[1] ?? 0);

    if (axisY <= -DIRECTION_DEADZONE) return "up";
    if (axisY >= DIRECTION_DEADZONE) return "down";
    if (axisX <= -DIRECTION_DEADZONE) return "left";
    if (axisX >= DIRECTION_DEADZONE) return "right";

    return null;
  }

  function poll(timestamp) {
    frameHandle = window.requestAnimationFrame(poll);

    if (!getEnabledValue(options.enabled)) {
      clearFocusedElement();
      previousButtons = [];
      lastDirection = null;
      controllerEngaged = false;
      graphState = {};
      graphId = null;
      return;
    }

    const gamepads = navigator.getGamepads?.() ?? [];
    const gamepad = gamepads.find(Boolean);
    if (!gamepad) {
      clearFocusedElement();
      previousButtons = [];
      lastDirection = null;
      controllerEngaged = false;
      return;
    }

    const currentButtons = gamepad.buttons.map((button) => Boolean(button?.pressed));

    const direction = readDirection(gamepad);
    const anyControllerInput =
      Boolean(direction) ||
      currentButtons.some(Boolean);
    let justEngagedThisFrame = false;
    if (!controllerEngaged && anyControllerInput) {
      controllerEngaged = true;
      justEngagedThisFrame = true;
      refreshFocusedElementClass();
    }
    if (
      controllerEngaged &&
      focusedElement instanceof HTMLElement &&
      !focusedElement.classList.contains("controller-nav-focused")
    ) {
      refreshFocusedElementClass();
    }
    if (!direction) {
      lastDirection = null;
      directionStartedAt = 0;
      lastDirectionRepeatAt = 0;
    } else if (direction !== lastDirection) {
      moveFocus(direction);
      lastDirection = direction;
      directionStartedAt = timestamp;
      lastDirectionRepeatAt = timestamp;
    } else if (
      timestamp - directionStartedAt >= INITIAL_REPEAT_DELAY &&
      timestamp - lastDirectionRepeatAt >= REPEAT_INTERVAL
    ) {
      moveFocus(direction);
      lastDirectionRepeatAt = timestamp;
    }

    if (!justEngagedThisFrame && currentButtons[0] && !previousButtons[0]) {
      activateFocusedElement();
    }

    if (currentButtons[1] && !previousButtons[1]) {
      handleBack();
    }

    if (currentButtons[4] && !previousButtons[4]) {
      scrollScope(-1);
    }

    if (currentButtons[5] && !previousButtons[5]) {
      scrollScope(1);
    }

    previousButtons = currentButtons;
  }

  function onKeyboardMouseInput(event) {
    if (!controllerEngaged) return;
    if (event?.type === "keydown") {
      const keyboardEvent = event;
      if (keyboardEvent.isComposing) return;
      if (keyboardEvent.repeat) return;
    }
    controllerEngaged = false;
    removeFocusClasses(focusedElement);
  }

  function onControllerFocusSync(event) {
    if (!controllerEngaged || !getEnabledValue(options.enabled)) return;
    const target = event?.detail?.element ?? event?.target;
    if (!isControllerFocusableElement(target)) return;
    applyFocus(target);
  }

  onMounted(() => {
    frameHandle = window.requestAnimationFrame(poll);
    window.addEventListener("pointerdown", onKeyboardMouseInput, true);
    window.addEventListener("mousemove", onKeyboardMouseInput, true);
    window.addEventListener("wheel", onKeyboardMouseInput, { passive: true, capture: true });
    window.addEventListener("keydown", onKeyboardMouseInput, true);
    window.addEventListener("controller-nav-sync-focus", onControllerFocusSync, true);
  });

  onBeforeUnmount(() => {
    if (frameHandle) {
      window.cancelAnimationFrame(frameHandle);
      frameHandle = 0;
    }
    clearFocusedElement();
    graphState = {};
    graphId = null;
    window.removeEventListener("pointerdown", onKeyboardMouseInput, true);
    window.removeEventListener("mousemove", onKeyboardMouseInput, true);
    window.removeEventListener("wheel", onKeyboardMouseInput, true);
    window.removeEventListener("keydown", onKeyboardMouseInput, true);
    window.removeEventListener("controller-nav-sync-focus", onControllerFocusSync, true);
  });

  watch(
    () => getScope(),
    () => {
      clearFocusedElement();
      graphState = {};
      graphId = null;
      window.requestAnimationFrame(() => {
        focusFirstAvailable();
      });
    }
  );

  watch(
    () => graphKeyFor(getGraph()),
    () => {
      clearFocusedElement();
      graphState = {};
      graphId = null;
      window.requestAnimationFrame(() => {
        focusFirstAvailable();
      });
    }
  );

  watch(
    () => getEnabledValue(options.enabled),
    (enabled) => {
      if (!enabled) {
        clearFocusedElement();
        graphState = {};
        graphId = null;
      } else {
        window.requestAnimationFrame(() => {
          focusFirstAvailable();
        });
      }
    },
    { immediate: true }
  );
}
