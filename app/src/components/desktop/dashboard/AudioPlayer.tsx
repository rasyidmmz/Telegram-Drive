import { useCallback, useEffect, useRef, useState } from 'react';
import { ChevronLeft, ChevronRight, Maximize2, Minimize2, X } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { TelegramFile } from '../../../types';

interface StreamInfo {
    token: string;
    base_url: string;
}

interface AudioPlayerProps {
    file: TelegramFile;
    onClose: () => void;
    onNext?: () => void;
    onPrev?: () => void;
    currentIndex?: number;
    totalItems?: number;
    activeFolderId: number | null;
}

export function AudioPlayer({ file, onClose, onNext, onPrev, currentIndex, totalItems, activeFolderId }: AudioPlayerProps) {
    const [streamInfo, setStreamInfo] = useState<StreamInfo | null>(null);
    const [isFullscreen, setIsFullscreen] = useState(false);
    const containerRef = useRef<HTMLDivElement>(null);
    const audioRef = useRef<HTMLAudioElement>(null);

    const toggleFullscreen = useCallback(async () => {
        try {
            const win = getCurrentWindow();
            const fullscreen = await win.isFullscreen();
            await win.setFullscreen(!fullscreen);
            setIsFullscreen(!fullscreen);
        } catch {
            const element = containerRef.current;
            if (!element) return;
            if (document.fullscreenElement) {
                await document.exitFullscreen();
            } else {
                await element.requestFullscreen();
            }
        }
    }, []);

    useEffect(() => {
        invoke<StreamInfo>('cmd_get_stream_info').then(setStreamInfo).catch(() => {});
    }, []);

    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            const target = event.target as HTMLElement;
            if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) return;

            const key = event.key.toLowerCase();
            if (event.key === 'ArrowRight' || key === 'l') {
                event.preventDefault();
                onNext?.();
            } else if (event.key === 'ArrowLeft' || key === 'j') {
                event.preventDefault();
                onPrev?.();
            } else if (event.key === 'Escape') {
                event.preventDefault();
                onClose();
            } else if (key === 'f') {
                event.preventDefault();
                void toggleFullscreen();
            } else if (key === 'm' && audioRef.current) {
                event.preventDefault();
                audioRef.current.muted = !audioRef.current.muted;
            } else if (event.key === ' ' && audioRef.current) {
                event.preventDefault();
                if (audioRef.current.paused) {
                    void audioRef.current.play();
                } else {
                    audioRef.current.pause();
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onClose, onNext, onPrev, toggleFullscreen]);

    const folderId = activeFolderId !== null ? activeFolderId.toString() : 'home';
    const streamUrl = streamInfo
        ? `${streamInfo.base_url}/stream/${folderId}/${file.id}?token=${streamInfo.token}`
        : null;

    return (
        <div className={`fixed inset-0 z-[200] bg-black/90 animate-in fade-in duration-200 ${isFullscreen ? 'p-0' : 'flex items-center justify-center p-4 backdrop-blur-md'}`} onClick={onClose}>
            <div ref={containerRef} className={`relative ${isFullscreen ? 'w-full h-full' : 'w-full max-w-6xl flex flex-col items-center'}`} onClick={(event) => event.stopPropagation()}>
                <div className={`absolute z-30 flex items-center gap-2 ${isFullscreen ? 'top-4 right-4' : '-top-12 right-0'}`}>
                    <button onClick={() => void toggleFullscreen()} className="w-10 h-10 flex items-center justify-center text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all" title={isFullscreen ? 'Exit fullscreen (F)' : 'Fullscreen (F)'}>
                        {isFullscreen ? <Minimize2 className="w-5 h-5" /> : <Maximize2 className="w-5 h-5" />}
                    </button>
                    <button onClick={onClose} className="w-10 h-10 flex items-center justify-center text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all" title="Close (Esc)">
                        <X className="w-5 h-5" />
                    </button>
                </div>

                <button onClick={onPrev} className={`absolute top-1/2 -translate-y-1/2 p-2 text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all z-10 ${isFullscreen ? 'left-4' : 'left-2'}`} title="Previous (ArrowLeft / J)">
                    <ChevronLeft className="w-6 h-6" />
                </button>
                <button onClick={onNext} className={`absolute top-1/2 -translate-y-1/2 p-2 text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all z-10 ${isFullscreen ? 'right-4' : 'right-2'}`} title="Next (ArrowRight / L)">
                    <ChevronRight className="w-6 h-6" />
                </button>

                <div className={`bg-black overflow-hidden flex items-center justify-center ${isFullscreen ? 'w-full h-full rounded-none' : 'w-full aspect-video rounded-xl shadow-2xl ring-1 ring-white/10'}`}>
                    {!streamUrl ? (
                        <div className="flex flex-col items-center gap-4 text-white">
                            <div className="w-10 h-10 border-4 border-telegram-primary border-t-transparent rounded-full animate-spin" />
                            <p>Preparing stream...</p>
                        </div>
                    ) : (
                        <div className="w-full h-full flex flex-col items-center justify-center bg-gradient-to-br from-telegram-primary/20 to-black">
                            <div className="w-32 h-32 rounded-full bg-telegram-surface flex items-center justify-center mb-8 shadow-xl animate-pulse-slow">
                                <svg xmlns="http://www.w3.org/2000/svg" className="w-12 h-12 text-telegram-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" /></svg>
                            </div>
                            <audio ref={audioRef} src={streamUrl} controls autoPlay className="w-full max-w-md" />
                        </div>
                    )}
                </div>

                {!isFullscreen && (
                    <div className="mt-4 text-center text-white">
                        <h3 className="text-lg font-medium truncate px-4">{file.name}</h3>
                        <p className="text-sm text-white/50">
                            Streaming from Telegram Drive
                            {typeof currentIndex === 'number' && typeof totalItems === 'number' && totalItems > 0 && (
                                <span className="ml-2">• {currentIndex + 1}/{totalItems}</span>
                            )}
                        </p>
                    </div>
                )}
            </div>
        </div>
    );
}
