import { AnimatePresence, motion } from 'framer-motion';
import { AlertTriangle, Copy, ScrollText, Trash2, X } from 'lucide-react';
import { toast } from 'sonner';
import { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { clearErrorLogs, ErrorLogEntry, useErrorLogs } from '../../../errorLogs';

interface LogsModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export function LogsModal({ isOpen, onClose }: LogsModalProps) {
    const frontendLogs = useErrorLogs();
    const [backendLogs, setBackendLogs] = useState<ErrorLogEntry[]>([]);
    const logs = useMemo(
        () => [...backendLogs, ...frontendLogs].sort((a, b) => Date.parse(b.time) - Date.parse(a.time)),
        [backendLogs, frontendLogs],
    );

    useEffect(() => {
        if (!isOpen) return;
        invoke<Array<Omit<ErrorLogEntry, 'id'>>>('cmd_get_transfer_logs')
            .then(items => {
                setBackendLogs(items.map((item, index) => ({
                    id: `backend-${item.time}-${index}`,
                    ...item,
                })));
            })
            .catch(e => {
                toast.error(`Failed to load backend logs: ${e}`);
            });
    }, [isOpen]);

    const copyLogs = async () => {
        const text = logs.map(formatEntry).join('\n\n---\n\n');
        await navigator.clipboard.writeText(text || 'No logs');
        toast.success('Logs copied');
    };

    const clearLogs = async () => {
        clearErrorLogs();
        setBackendLogs([]);
        try {
            await invoke('cmd_clear_transfer_logs');
        } catch (e) {
            toast.error(`Failed to clear backend logs: ${e}`);
        }
    };

    return (
        <AnimatePresence>
            {isOpen && (
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="fixed inset-0 bg-black/50 backdrop-blur-sm z-[200] flex items-center justify-center p-4"
                    onClick={onClose}
                >
                    <motion.div
                        initial={{ scale: 0.95, opacity: 0 }}
                        animate={{ scale: 1, opacity: 1 }}
                        exit={{ scale: 0.95, opacity: 0 }}
                        transition={{ type: 'spring', damping: 25, stiffness: 300 }}
                        onClick={e => e.stopPropagation()}
                        className="w-full max-w-3xl max-h-[82vh] bg-telegram-surface border border-telegram-border rounded-xl shadow-2xl overflow-hidden flex flex-col"
                    >
                        <div className="px-5 py-4 border-b border-telegram-border flex items-center justify-between bg-telegram-hover/40">
                            <div className="flex items-center gap-3">
                                <div className="w-9 h-9 rounded-lg bg-red-500/10 border border-red-500/20 flex items-center justify-center">
                                    <ScrollText className="w-5 h-5 text-red-400" />
                                </div>
                                <div>
                                    <h2 className="text-base font-semibold text-telegram-text">Logs</h2>
                                    <p className="text-xs text-telegram-subtext">{logs.length} error{logs.length === 1 ? '' : 's'}</p>
                                </div>
                            </div>
                            <button
                                onClick={onClose}
                                className="p-2 rounded-lg text-telegram-subtext hover:text-telegram-text hover:bg-telegram-hover transition"
                                title="Close"
                            >
                                <X className="w-5 h-5" />
                            </button>
                        </div>

                        <div className="p-4 flex-1 overflow-y-auto">
                            {logs.length === 0 ? (
                                <div className="h-48 flex flex-col items-center justify-center text-center text-telegram-subtext">
                                    <AlertTriangle className="w-8 h-8 mb-3 text-telegram-subtext/60" />
                                    <p className="text-sm">No errors recorded</p>
                                </div>
                            ) : (
                                <div className="space-y-3">
                                    {logs.map(log => (
                                        <article key={log.id} className="rounded-lg border border-telegram-border bg-telegram-hover/40 p-3">
                                            <div className="flex items-start justify-between gap-3">
                                                <div className="min-w-0">
                                                    <div className="flex items-center gap-2 mb-1">
                                                        <span className="text-xs font-medium text-red-400">{log.source}</span>
                                                        {log.category && (
                                                            <span className="rounded bg-telegram-border px-1.5 py-0.5 text-[10px] text-telegram-subtext">
                                                                {log.category}
                                                            </span>
                                                        )}
                                                        <span className="text-[11px] text-telegram-subtext">{new Date(log.time).toLocaleString()}</span>
                                                    </div>
                                                    <p className="text-sm text-telegram-text break-words">{log.message}</p>
                                                </div>
                                            </div>
                                            {log.details && (
                                                <pre className="mt-3 max-h-48 overflow-auto rounded-md bg-black/20 border border-telegram-border p-3 text-[11px] leading-relaxed text-telegram-subtext whitespace-pre-wrap break-words">
                                                    {log.details}
                                                </pre>
                                            )}
                                        </article>
                                    ))}
                                </div>
                            )}
                        </div>

                        <div className="px-5 py-3 border-t border-telegram-border flex items-center justify-between">
                            <button
                                onClick={clearLogs}
                                disabled={logs.length === 0}
                                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs text-telegram-subtext hover:text-red-400 hover:bg-red-500/10 transition font-medium disabled:opacity-40 disabled:pointer-events-none"
                            >
                                <Trash2 className="w-3.5 h-3.5" />
                                Clear
                            </button>
                            <button
                                onClick={copyLogs}
                                disabled={logs.length === 0}
                                className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-xs font-medium bg-telegram-primary text-white hover:bg-telegram-primary/90 transition disabled:opacity-40 disabled:pointer-events-none"
                            >
                                <Copy className="w-3.5 h-3.5" />
                                Copy
                            </button>
                        </div>
                    </motion.div>
                </motion.div>
            )}
        </AnimatePresence>
    );
}

function formatEntry(log: ErrorLogEntry) {
    const source = log.category ? `${log.source} / ${log.category}` : log.source;
    return [
        `[${new Date(log.time).toLocaleString()}] ${source}`,
        log.message,
        log.details,
    ].filter(Boolean).join('\n');
}
