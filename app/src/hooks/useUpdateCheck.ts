import { useState, useEffect, useCallback } from 'react';
import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

interface UpdateState {
    checking: boolean;
    available: boolean;
    downloading: boolean;
    installing: boolean;
    restarting: boolean;
    progress: number;
    error: string | null;
    version: string | null;
}

interface UseUpdateCheckOptions {
    autoCheck?: boolean;
}

function updateErrorMessage(err: unknown, fallback: string) {
    if (err instanceof Error && err.message) return err.message;
    if (typeof err === 'string' && err.trim()) return err;
    if (err && typeof err === 'object' && 'message' in err) {
        const message = String((err as { message?: unknown }).message || '').trim();
        if (message) return message;
    }
    return fallback;
}

export function useUpdateCheck(options: UseUpdateCheckOptions = {}) {
    const { autoCheck = true } = options;
    const [state, setState] = useState<UpdateState>({
        checking: false,
        available: false,
        downloading: false,
        installing: false,
        restarting: false,
        progress: 0,
        error: null,
        version: null,
    });
    const [update, setUpdate] = useState<Update | null>(null);

    const checkForUpdates = useCallback(async (): Promise<Update | null | undefined> => {
        setState(s => ({ ...s, checking: true, error: null }));
        try {
            const updateInfo = await check();
            if (updateInfo) {
                setUpdate(updateInfo);
                setState(s => ({
                    ...s,
                    checking: false,
                    available: true,
                    version: updateInfo.version,
                }));
                return updateInfo;
            } else {
                setState(s => ({ ...s, checking: false, available: false, version: null }));
                return null;
            }
        } catch (err: unknown) {
            const message = updateErrorMessage(err, 'Failed to check for updates');
            setState(s => ({
                ...s,
                checking: false,
                available: false,
                error: message,
            }));
            return undefined;
        }
    }, []);

    const downloadAndInstall = useCallback(async () => {
        if (!update) return;

        setState(s => ({ ...s, downloading: true, installing: false, restarting: false, error: null, progress: 0 }));
        let downloaded = 0;
        let contentLength = 0;

        try {
            await update.downloadAndInstall((event) => {
                if (event.event === 'Started') {
                    const data = event.data as { contentLength?: number };
                    contentLength = data.contentLength || 0;
                } else if (event.event === 'Progress') {
                    const data = event.data as { chunkLength?: number };
                    downloaded += data.chunkLength || 0;
                    if (contentLength > 0) {
                        const pct = Math.round((downloaded / contentLength) * 100);
                        setState(s => ({ ...s, progress: Math.min(pct, 100) }));
                    }
                } else if (event.event === 'Finished') {
                    setState(s => ({ ...s, downloading: false, installing: true, progress: 100 }));
                }
            });

        } catch (err: unknown) {
            const message = updateErrorMessage(err, 'Failed to install update');
            setState(s => ({
                ...s,
                downloading: false,
                installing: false,
                restarting: false,
                error: message,
            }));
            return;
        }

        setState(s => ({ ...s, installing: false, restarting: true, progress: 100 }));
        try {
            await relaunch();
        } catch (err: unknown) {
            const message = updateErrorMessage(err, 'Update installed, but Teledrive could not restart automatically');
            setState(s => ({
                ...s,
                restarting: false,
                error: message,
            }));
        }
    }, [update]);

    const dismissUpdate = useCallback(() => {
        setState(s => ({ ...s, available: false, error: null }));
        setUpdate(null);
    }, []);

    useEffect(() => {
        if (!autoCheck) return;
        const timer = setTimeout(() => {
            checkForUpdates().catch(console.error);
        }, 5000);
        return () => clearTimeout(timer);
    }, [autoCheck, checkForUpdates]);

    return {
        ...state,
        checkForUpdates,
        downloadAndInstall,
        dismissUpdate,
    };
}
