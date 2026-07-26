import { RefObject, useEffect, useRef } from 'react';

const focusableSelector = [
    'button:not([disabled])', 'a[href]', 'input:not([disabled])',
    'select:not([disabled])', 'textarea:not([disabled])', '[tabindex]:not([tabindex="-1"])',
].join(',');

export function useModalDialog(
    isOpen: boolean,
    onClose: () => void,
    initialFocus: RefObject<HTMLElement | null>,
) {
    const dialogRef = useRef<HTMLDivElement>(null);
    const returnFocusRef = useRef<HTMLElement | null>(null);

    useEffect(() => {
        if (!isOpen) return;

        returnFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
        requestAnimationFrame(() => (initialFocus.current ?? dialogRef.current)?.focus());

        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                event.preventDefault();
                onClose();
                return;
            }
            if (event.key !== 'Tab' || !dialogRef.current) return;

            const controls = Array.from(dialogRef.current.querySelectorAll<HTMLElement>(focusableSelector));
            if (controls.length === 0) {
                event.preventDefault();
                dialogRef.current.focus();
                return;
            }
            const first = controls[0];
            const last = controls[controls.length - 1];
            if (event.shiftKey && document.activeElement === first) {
                event.preventDefault();
                last.focus();
            } else if (!event.shiftKey && document.activeElement === last) {
                event.preventDefault();
                first.focus();
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => {
            document.removeEventListener('keydown', handleKeyDown);
            returnFocusRef.current?.focus();
        };
    }, [initialFocus, isOpen, onClose]);

    return dialogRef;
}
