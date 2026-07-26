import { TelegramFile } from '../types';

export interface WatchHistoryEntry {
    id: string;
    file_id: number;
    file_name: string;
    folder_id: number | null;
    file_size: number;
    timestamp: string; // ISO string
    last_position_secs?: number;
    total_duration_secs?: number;
    status: 'started' | 'playing' | 'completed' | 'paused';
    quality_tag?: string;
}

export interface WatchLogEvent {
    id: string;
    timestamp: string;
    file_name: string;
    event_type: 'PLAY_START' | 'PAUSE' | 'RESUME' | 'SUBTITLE_GEN' | 'SEEK' | 'COMPLETED' | 'ERROR';
    details: string;
}

const STORAGE_KEY_HISTORY = 'telestash_recent_watch_v1';
const STORAGE_KEY_LOGS = 'telestash_watch_logs_v1';
const MAX_LOG_ENTRIES = 500;
const MAX_HISTORY_ENTRIES = 50;

/**
 * Get all recent watch history entries sorted by latest first
 */
export function getRecentWatchHistory(): WatchHistoryEntry[] {
    try {
        const raw = localStorage.getItem(STORAGE_KEY_HISTORY);
        if (!raw) return [];
        const items: WatchHistoryEntry[] = JSON.parse(raw);
        return items.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
    } catch {
        return [];
    }
}

/**
 * Record a playback start or update event
 */
export function recordWatchEvent(
    file: TelegramFile,
    status: WatchHistoryEntry['status'] = 'started',
    qualityTag?: string,
    lastPosSecs?: number,
    totalDurationSecs?: number
): WatchHistoryEntry {
    const history = getRecentWatchHistory();
    const existingIndex = history.findIndex(item => item.file_id === file.id);
    
    const entry: WatchHistoryEntry = {
        id: existingIndex >= 0 ? history[existingIndex].id : `watch_${Date.now()}_${file.id}`,
        file_id: file.id,
        file_name: file.name,
        folder_id: file.folder_id ?? null,
        file_size: file.size,
        timestamp: new Date().toISOString(),
        status,
        quality_tag: qualityTag || existingIndex >= 0 ? history[existingIndex]?.quality_tag : undefined,
        last_position_secs: lastPosSecs ?? (existingIndex >= 0 ? history[existingIndex]?.last_position_secs : 0),
        total_duration_secs: totalDurationSecs ?? (existingIndex >= 0 ? history[existingIndex]?.total_duration_secs : 0)
    };

    if (existingIndex >= 0) {
        history[existingIndex] = entry;
    } else {
        history.unshift(entry);
    }

    // Limit max entries
    const trimmed = history.slice(0, MAX_HISTORY_ENTRIES);
    try {
        localStorage.setItem(STORAGE_KEY_HISTORY, JSON.stringify(trimmed));
    } catch (e) {
        console.error('Failed to save watch history:', e);
    }

    // Also add to watch logs
    addWatchLog(
        file.name,
        status === 'started' ? 'PLAY_START' : status === 'completed' ? 'COMPLETED' : 'RESUME',
        `Playback ${status} for ${file.name}${qualityTag ? ` [${qualityTag}]` : ''}`
    );

    return entry;
}

/**
 * Remove an entry from recent watch history
 */
export function removeWatchEntry(fileId: number): void {
    const history = getRecentWatchHistory().filter(item => item.file_id !== fileId);
    try {
        localStorage.setItem(STORAGE_KEY_HISTORY, JSON.stringify(history));
    } catch (e) {
        console.error('Failed to update watch history:', e);
    }
}

/**
 * Clear all recent watch history
 */
export function clearWatchHistory(): void {
    try {
        localStorage.removeItem(STORAGE_KEY_HISTORY);
    } catch (e) {
        console.error('Failed to clear watch history:', e);
    }
}

/**
 * Get all watch activity logs
 */
export function getWatchLogs(): WatchLogEvent[] {
    try {
        const raw = localStorage.getItem(STORAGE_KEY_LOGS);
        if (!raw) return [];
        const logs: WatchLogEvent[] = JSON.parse(raw);
        return logs.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
    } catch {
        return [];
    }
}

/**
 * Add a new log entry to Watch History Logs (separated from error logs)
 */
export function addWatchLog(fileName: string, eventType: WatchLogEvent['event_type'], details: string): void {
    const logs = getWatchLogs();
    const newLog: WatchLogEvent = {
        id: `wlog_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
        timestamp: new Date().toISOString(),
        file_name: fileName,
        event_type: eventType,
        details
    };

    logs.unshift(newLog);
    const trimmed = logs.slice(0, MAX_LOG_ENTRIES);

    try {
        localStorage.setItem(STORAGE_KEY_LOGS, JSON.stringify(trimmed));
    } catch (e) {
        console.error('Failed to save watch log:', e);
    }
}

/**
 * Clear all watch logs
 */
export function clearWatchLogs(): void {
    try {
        localStorage.removeItem(STORAGE_KEY_LOGS);
    } catch (e) {
        console.error('Failed to clear watch logs:', e);
    }
}

/**
 * Export watch logs as a formatted JSON or text string
 */
export function exportWatchLogsText(): string {
    const logs = getWatchLogs();
    return logs.map(l => `[${l.timestamp}] [${l.event_type}] ${l.file_name} -> ${l.details}`).join('\n');
}
