import { useEffect, useState } from 'react';
import { X, ChevronLeft, ChevronRight } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { TelegramFile } from '../../../types';
import { isVideoFile, isAudioFile } from '../../../utils';
import { toast } from 'sonner';

interface StreamInfo {
    token: string;
    base_url: string;
}

interface MediaPlayerProps {
    file: TelegramFile;
    onClose: () => void;
    onNext?: () => void;
    onPrev?: () => void;
    currentIndex?: number;
    totalItems?: number;
    activeFolderId: number | null;
}

export function MediaPlayer({ file, onClose, onNext, onPrev, currentIndex, totalItems, activeFolderId }: MediaPlayerProps) {
    const [streamInfo, setStreamInfo] = useState<StreamInfo | null>(null);
    const [isPlayingInMpv, setIsPlayingInMpv] = useState(false);
    const [mpvError, setMpvError] = useState<string | null>(null);

    // Get stream info on mount
    useEffect(() => {
        invoke<StreamInfo>('cmd_get_stream_info').then(setStreamInfo).catch(() => {});
    }, []);

    const folderIdParam = activeFolderId !== null ? activeFolderId.toString() : 'home';
    const streamUrl = streamInfo
        ? `${streamInfo.base_url}/stream/${folderIdParam}/${file.id}?token=${streamInfo.token}`
        : null;

    const isVideo = isVideoFile(file.name);
    const isAudio = isAudioFile(file.name);
    const isMedia = isVideo || isAudio;

    // Reset MPV state when file changes (playlist navigation)
    useEffect(() => {
        setIsPlayingInMpv(false);
        setMpvError(null);
    }, [file.id]);

    // Automatically trigger MPV launch when streamUrl is ready
    useEffect(() => {
        if (isMedia && streamUrl && !isPlayingInMpv && !mpvError) {
            invoke('cmd_play_in_mpv', { url: streamUrl, messageId: file.id, folderId: file.folder_id })
                .then(() => {
                    setIsPlayingInMpv(true);
                })
                .catch((err) => {
                    console.error('Failed to play in MPV:', err);
                    const errMsg = err?.toString() || 'Gagal membuka MPV';
                    setMpvError(errMsg);
                    toast.error(`Gagal memutar di MPV: ${errMsg}`);
                });
        }
    }, [isMedia, streamUrl, isPlayingInMpv, mpvError, file.id, file.folder_id]);

    // Handle keyboard shortcuts (Left/Right arrow keys for navigation, Esc for close)
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            const target = e.target as HTMLElement;
            if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
                return;
            }

            const key = e.key.toLowerCase();
            if (e.key === 'ArrowRight' || key === 'l') {
                e.preventDefault();
                onNext?.();
            } else if (e.key === 'ArrowLeft' || key === 'j') {
                e.preventDefault();
                onPrev?.();
            } else if (e.key === 'Escape') {
                e.preventDefault();
                onClose();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onClose, onNext, onPrev]);

    return (
        <div className="fixed inset-0 z-[200] bg-black/90 flex items-center justify-center p-4 backdrop-blur-md" onClick={onClose}>
            <div className="relative w-full max-w-lg text-center p-8 bg-telegram-surface border border-telegram-border/60 rounded-2xl shadow-2xl flex flex-col items-center gap-6" onClick={e => e.stopPropagation()}>
                {/* Close Button */}
                <div className="absolute top-4 right-4">
                    <button
                        onClick={onClose}
                        className="w-10 h-10 flex items-center justify-center text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all"
                        title="Tutup (Esc)"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Playlist Navigation Buttons */}
                {onPrev && (
                    <button
                        onClick={onPrev}
                        className="absolute left-4 top-1/2 -translate-y-1/2 p-2 text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all z-10"
                        title="Sebelumnya (ArrowLeft / J)"
                    >
                        <ChevronLeft className="w-6 h-6" />
                    </button>
                )}

                {onNext && (
                    <button
                        onClick={onNext}
                        className="absolute right-4 top-1/2 -translate-y-1/2 p-2 text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all z-10"
                        title="Berikutnya (ArrowRight / L)"
                    >
                        <ChevronRight className="w-6 h-6" />
                    </button>
                )}

                {/* Media Icon & Status */}
                {!streamUrl ? (
                    <div className="flex flex-col items-center gap-4 py-6">
                        <div className="w-10 h-10 border-4 border-telegram-primary border-t-transparent rounded-full animate-spin"></div>
                        <p className="text-white/80">Menghubungkan stream...</p>
                    </div>
                ) : mpvError ? (
                    <div className="flex flex-col items-center gap-4 py-4 text-center">
                        <div className="w-16 h-16 rounded-full bg-red-500/10 flex items-center justify-center text-red-500 border border-red-500/20">
                            <X className="w-8 h-8" />
                        </div>
                        <h3 className="text-lg font-bold text-white">Gagal Memutar Media</h3>
                        <p className="text-sm text-white/60 max-w-sm">{mpvError}</p>
                    </div>
                ) : (
                    <>
                        <div className="w-20 h-20 rounded-full bg-telegram-primary/10 flex items-center justify-center text-telegram-primary border border-telegram-primary/25 animate-pulse">
                            {isVideo ? (
                                <svg xmlns="http://www.w3.org/2000/svg" className="w-10 h-10 fill-current" viewBox="0 0 24 24"><polygon points="6 3 20 12 6 21 6 3"/></svg>
                            ) : (
                                <svg xmlns="http://www.w3.org/2000/svg" className="w-10 h-10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" /></svg>
                            )}
                        </div>

                        <div>
                            <h3 className="text-xl font-bold text-white mb-2 line-clamp-2 px-6">{file.name}</h3>
                            <p className="text-sm text-white/60">
                                {isPlayingInMpv 
                                    ? `Memutar ${isVideo ? 'video' : 'audio'} di MPV secara native...` 
                                    : `Membuka ${isVideo ? 'video' : 'audio'} di MPV...`}
                            </p>
                        </div>

                        <div className="flex flex-col gap-2 w-full mt-2">
                            <button
                                onClick={() => {
                                    if (streamUrl) {
                                        invoke('cmd_play_in_mpv', { url: streamUrl, messageId: file.id, folderId: file.folder_id }).catch(err => {
                                            toast.error(`Gagal membuka kembali MPV: ${err}`);
                                        });
                                    }
                                }}
                                className="w-full py-3 bg-telegram-primary text-black rounded-xl hover:shadow-lg hover:shadow-telegram-primary/20 transition-all text-sm font-semibold"
                            >
                                Buka Kembali di MPV
                            </button>
                            <button
                                onClick={onClose}
                                className="w-full py-3 bg-white/5 hover:bg-white/10 text-white border border-white/10 rounded-xl transition-all text-sm font-semibold"
                            >
                                Tutup Pemutar
                            </button>
                        </div>
                    </>
                )}

                {/* Playlist Counter */}
                {typeof currentIndex === 'number' && typeof totalItems === 'number' && totalItems > 0 && (
                    <div className="text-[11px] text-white/40 select-none">
                        Berkas {currentIndex + 1} dari {totalItems}
                    </div>
                )}
            </div>
        </div>
    );
}
