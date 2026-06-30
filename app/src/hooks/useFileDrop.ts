/**
 * Simple file drop hook.
 *
 * External OS file drops are disabled. Internal file moves still use component
 * drop handlers.
 */
export function useFileDrop() {
    return { isDragging: false };
}
