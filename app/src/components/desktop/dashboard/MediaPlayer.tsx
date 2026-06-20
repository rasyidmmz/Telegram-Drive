import { useEffect, useState, useRef, useCallback } from 'react';
import { X, ChevronLeft, ChevronRight, Maximize2, Minimize2, Subtitles, ExternalLink } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { toast } from 'sonner';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { TelegramFile } from '../../../types';
import { isVideoFile, isAudioFile } from '../../../utils';
import { AdaptiveMediaPlayer } from './AdaptiveMediaPlayer';

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

function isMp4Video(name: string): boolean {
    return name.toLowerCase().endsWith('.mp4');
}

// ── SRT to WebVTT Converter ──────────────────────────────────────────
function srtToVtt(srt: string): string {
    let vtt = 'WEBVTT\n\n';
    const lines = srt.replace(/\r\n/g, '\n').split('\n');
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        if (line.match(/^\d+$/)) {
            vtt += line + '\n';
        } else if (line.match(/^\d{2}:\d{2}:\d{2},\d{3}/)) {
            vtt += line.replace(/,/g, '.') + '\n';
        } else {
            vtt += line + '\n';
        }
    }
    return vtt;
}

export function MediaPlayer({ file, onClose, onNext, onPrev, currentIndex, totalItems, activeFolderId }: MediaPlayerProps) {
    const [streamInfo, setStreamInfo] = useState<StreamInfo | null>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [isFullscreen, setIsFullscreen] = useState(false);

    const toggleFullscreen = useCallback(async () => {
        try {
            const win = getCurrentWindow();
            const fs = await win.isFullscreen();
            await win.setFullscreen(!fs);
            setIsFullscreen(!fs);
        } catch {
            // Not running in Tauri — fall back to webview fullscreen
            const el = containerRef.current;
            if (!el) return;
            if (document.fullscreenElement) {
                document.exitFullscreen().catch(() => {});
            } else {
                el.requestFullscreen().catch(() => {});
            }
        }
    }, []);

    // Sync isFullscreen when OS changes fullscreen (e.g. Escape / green button)
    useEffect(() => {
        let mounted = true;
        let unlistenFn: (() => void) | undefined;
        getCurrentWindow().onResized(async () => {
            if (!mounted) return;
            try {
                const fs = await getCurrentWindow().isFullscreen();
                setIsFullscreen(fs);
            } catch {}
        }).then(fn => { if (mounted) unlistenFn = fn; });
        return () => {
            mounted = false;
            unlistenFn?.();
        };
    }, []);

    useEffect(() => {
        invoke<StreamInfo>('cmd_get_stream_info').then(setStreamInfo).catch(() => {});
    }, []);

    const folderIdParam = activeFolderId !== null ? activeFolderId.toString() : 'home';
    const streamUrl = streamInfo
        ? `${streamInfo.base_url}/stream/${folderIdParam}/${file.id}?token=${streamInfo.token}`
        : null;

    const isVideo = isVideoFile(file.name);
    const isAudio = isAudioFile(file.name);
    const isMp4 = isMp4Video(file.name);

    // ── Subtitle State ───────────────────────────────────────────────
    const [subtitleUrl, setSubtitleUrl] = useState<string | null>(null);
    const [subtitleEnabled, setSubtitleEnabled] = useState(true);

    const copyStreamUrlToClipboard = async () => {
        if (!streamUrl) return;
        try {
            await writeText(streamUrl);
            toast.success('Link aliran media disalin! Buka VLC lalu tekan Ctrl+N untuk memutar.', { duration: 5000 });
        } catch (err) {
            toast.error('Gagal menyalin link');
        }
    };

    useEffect(() => {
        if (!activeFolderId || !file.name || !streamInfo) return;
        
        let mounted = true;
        
        const fetchSubtitles = async () => {
            try {
                const srtName = file.name.replace(/\.[^/.]+$/, "") + ".srt";
                const files = await invoke<TelegramFile[]>('cmd_list_files', { folderId: activeFolderId, forceRefresh: false });
                const srtFile = files.find(f => f.name === srtName);
                
                if (srtFile && mounted) {
                    const url = `${streamInfo.base_url}/stream/${activeFolderId}/${srtFile.id}?token=${streamInfo.token}`;
                    const response = await fetch(url);
                    if (response.ok && mounted) {
                        const srtText = await response.text();
                        const vttText = srtToVtt(srtText);
                        const blob = new Blob([vttText], { type: 'text/vtt' });
                        const blobUrl = URL.createObjectURL(blob);
                        setSubtitleUrl(blobUrl);
                    }
                }
            } catch (e) {
                console.error("Failed to fetch subtitles:", e);
            }
        };

        fetchSubtitles();

        return () => {
            mounted = false;
        };
    }, [file.name, activeFolderId, streamInfo]);

    useEffect(() => {
        return () => {
            if (subtitleUrl) URL.revokeObjectURL(subtitleUrl);
        };
    }, [subtitleUrl]);

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
                return;
            }

            if (e.key === 'ArrowLeft' || key === 'j') {
                e.preventDefault();
                onPrev?.();
                return;
            }

            if (e.key === 'Escape') {
                e.preventDefault();
                onClose();
            }

            if (key === 'f') {
                e.preventDefault();
                toggleFullscreen();
            }

            if (key === 'm') {
                e.preventDefault();
                const video = document.querySelector('video');
                if (video) {
                    video.muted = !video.muted;
                }
            }

            if (e.key === ' ') {
                e.preventDefault();
                const video = document.querySelector('video');
                if (video) {
                    video.paused ? video.play().catch(() => {}) : video.pause();
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onClose, onNext, onPrev, toggleFullscreen]);

    // MP4 files: use adaptive streaming with quality controls + throttling
    if (isMp4 && streamUrl) {
        return (
            <AdaptiveMediaPlayer
                file={file}
                streamUrl={streamUrl}
                activeFolderId={activeFolderId}
                onClose={onClose}
                onNext={onNext}
                onPrev={onPrev}
                currentIndex={currentIndex}
                totalItems={totalItems}
            />
        );
    }

    return (
        <div className={`fixed inset-0 z-[200] bg-black/90 animate-in fade-in duration-200 ${isFullscreen ? 'p-0' : 'flex items-center justify-center p-4 backdrop-blur-md'}`} onClick={onClose}>
            <div ref={containerRef} className={`relative ${isFullscreen ? 'w-full h-full' : 'w-full max-w-6xl flex flex-col items-center'}`} onClick={e => e.stopPropagation()}>
                <div className={`absolute z-30 flex items-center gap-2 ${isFullscreen ? 'top-4 right-4' : '-top-12 right-0'}`}>
                    <button
                        onClick={toggleFullscreen}
                        className="w-10 h-10 flex items-center justify-center text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all"
                        title={isFullscreen ? 'Exit fullscreen (F)' : 'Fullscreen (F)'}
                    >
                        {isFullscreen ? <Minimize2 className="w-5 h-5" /> : <Maximize2 className="w-5 h-5" />}
                    </button>
                    <button
                        onClick={onClose}
                        className="w-10 h-10 flex items-center justify-center text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all"
                        title="Close (Esc)"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>
                <button
                    onClick={onPrev}
                    className={`absolute top-1/2 -translate-y-1/2 p-2 text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all z-10 ${isFullscreen ? 'left-4' : 'left-2'}`}
                    title="Previous (ArrowLeft / J)"
                >
                    <ChevronLeft className="w-6 h-6" />
                </button>

                <button
                    onClick={onNext}
                    className={`absolute top-1/2 -translate-y-1/2 p-2 text-white/50 hover:text-white bg-white/10 hover:bg-white/20 rounded-full transition-all z-10 ${isFullscreen ? 'right-4' : 'right-2'}`}
                    title="Next (ArrowRight / L)"
                >
                    <ChevronRight className="w-6 h-6" />
                </button>

                <div className={`bg-black overflow-hidden flex items-center justify-center ${isFullscreen ? 'w-full h-full rounded-none shadow-none ring-0' : 'w-full aspect-video rounded-xl shadow-2xl ring-1 ring-white/10'}`}>
                    {!streamUrl ? (
                        <div className="flex flex-col items-center gap-4 text-white">
                            <div className="w-10 h-10 border-4 border-telegram-primary border-t-transparent rounded-full animate-spin"></div>
                            <p>Preparing stream...</p>
                        </div>
                    ) : isVideo ? (
                        <video
                            src={streamUrl}
                            controls
                            controlsList="nodownload"
                            autoPlay
                            className="w-full h-full object-contain"
                        >
                            {subtitleUrl && subtitleEnabled && <track kind="subtitles" src={subtitleUrl} srcLang="en" label="English" default />}
                        </video>
                    ) : isAudio ? (
                        <div className="w-full h-full flex flex-col items-center justify-center bg-gradient-to-br from-telegram-primary/20 to-black">
                            <div className="w-32 h-32 rounded-full bg-telegram-surface flex items-center justify-center mb-8 shadow-xl animate-pulse-slow">
                                <svg xmlns="http://www.w3.org/2000/svg" className="w-12 h-12 text-telegram-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" /></svg>
                            </div>
                            <audio src={streamUrl} controls autoPlay className="w-full max-w-md" />
                        </div>
                    ) : (
                        <div className="text-white">Unsupported media type</div>
                    )}
                </div>

                {!isFullscreen && <div className="mt-4 flex items-center justify-between w-full max-w-4xl px-4">
                    <div className="flex-1 text-center pl-[120px] pr-4">
                        <h3 className="text-lg font-medium text-white truncate px-4">{file.name}</h3>
                        <p className="text-sm text-white/50">
                            Streaming from Telegram Drive
                            {typeof currentIndex === 'number' && typeof totalItems === 'number' && totalItems > 0 && (
                                <span className="ml-2">• {currentIndex + 1}/{totalItems}</span>
                            )}
                        </p>
                    </div>
                    {/* Controls */}
                    <div className="flex items-center gap-2 pr-4 w-[120px] justify-end flex-shrink-0">
                        <button
                            onClick={copyStreamUrlToClipboard}
                            className="p-2 rounded bg-white/5 hover:bg-white/10 transition-colors text-white/60 hover:text-white"
                            title="Buka di VLC / Pemutar Eksternal"
                        >
                            <ExternalLink className="w-5 h-5" />
                        </button>
                        {subtitleUrl && (
                            <button
                                onClick={() => setSubtitleEnabled(!subtitleEnabled)}
                                className={`p-2 rounded bg-white/5 hover:bg-white/10 transition-colors ${subtitleEnabled ? 'text-white' : 'text-white/40'}`}
                                title={subtitleEnabled ? 'Disable Subtitles' : 'Enable Subtitles'}
                            >
                                <Subtitles className="w-5 h-5" />
                            </button>
                        )}
                    </div>
                </div>}

                {/* Keyboard shortcut hints */}
                {!isFullscreen && <div className="mt-2 flex items-center gap-4 text-[10px] text-white/25 select-none">
                    <span className="flex items-center gap-1">
                        <kbd className="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[9px] font-mono">← →</kbd> Navigate
                    </span>
                    <span className="flex items-center gap-1">
                        <kbd className="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[9px] font-mono">Space</kbd> Play/Pause
                    </span>
                    <span className="flex items-center gap-1">
                        <kbd className="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[9px] font-mono">F</kbd> Fullscreen
                    </span>
                    <span className="flex items-center gap-1">
                        <kbd className="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[9px] font-mono">Esc</kbd> Close
                    </span>
                    <span className="flex items-center gap-1">
                        <kbd className="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[9px] font-mono">M</kbd> Mute
                    </span>
                </div>}
            </div>
        </div>
    );
}
