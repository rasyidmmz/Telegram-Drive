import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { VideoMetadata } from '../types';

const METADATA_STALE_TIME = 30 * 60 * 1000; // 30 minutes — metadata rarely changes

/**
 * Fetches MP4/MKV video metadata (duration, resolution) from the Rust backend.
 * Only fires for .mp4/.mkv files; returns null for non-video files.
 * Results are cached by React Query for 30 minutes.
 */
export function useVideoMetadata(
    messageId: number,
    folderId: number | null,
    fileName: string,
) {
    const lowerName = fileName.toLowerCase();
    const isSupportedVideo = lowerName.endsWith('.mp4') || lowerName.endsWith('.mkv');

    return useQuery({
        queryKey: ['video-metadata', folderId, messageId],
        queryFn: async (): Promise<VideoMetadata | null> => {
            if (!isSupportedVideo) return null;
            try {
                return await invoke<VideoMetadata>('cmd_get_video_metadata', {
                    messageId,
                    folderId,
                });
            } catch {
                // Metadata unavailable (moov-at-end, unsupported video, network error, etc.)
                return null;
            }
        },
        enabled: isSupportedVideo,
        staleTime: METADATA_STALE_TIME,
        retry: 1,
    });
}
