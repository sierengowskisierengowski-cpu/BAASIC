import { useCallback, useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { RepeatMode, Track } from "../types";
import { api } from "../lib/api";

function shuffleArray<T>(arr: T[]): T[] {
  const copy = [...arr];
  for (let i = copy.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [copy[i], copy[j]] = [copy[j], copy[i]];
  }
  return copy;
}

export function usePlayer() {
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [queue, setQueue] = useState<Track[]>([]);
  const [queueIndex, setQueueIndex] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [shuffle, setShuffle] = useState(false);
  const [repeat, setRepeat] = useState<RepeatMode>("off");
  const [volume, setVolume] = useState(0.85);
  const [muted, setMuted] = useState(false);
  const [position, setPosition] = useState(0);
  const [duration, setDuration] = useState(0);
  const [shuffledOrder, setShuffledOrder] = useState<number[]>([]);
  const recordedRef = useRef<number | null>(null);

  const currentTrack = queue[queueIndex] ?? null;

  useEffect(() => {
    const audio = new Audio();
    audioRef.current = audio;

    const onTime = () => setPosition(audio.currentTime);
    const onMeta = () => setDuration(audio.duration || 0);
    const onEnd = () => handleEndedRef.current();
    const onPlay = () => setIsPlaying(true);
    const onPause = () => setIsPlaying(false);

    audio.addEventListener("timeupdate", onTime);
    audio.addEventListener("loadedmetadata", onMeta);
    audio.addEventListener("ended", onEnd);
    audio.addEventListener("play", onPlay);
    audio.addEventListener("pause", onPause);

    return () => {
      audio.pause();
      audio.removeEventListener("timeupdate", onTime);
      audio.removeEventListener("loadedmetadata", onMeta);
      audio.removeEventListener("ended", onEnd);
      audio.removeEventListener("play", onPlay);
      audio.removeEventListener("pause", onPause);
    };
  }, []);

  const handleEndedRef = useRef(() => {});
  handleEndedRef.current = () => {
    if (repeat === "one") {
      const audio = audioRef.current;
      if (audio) {
        audio.currentTime = 0;
        void audio.play();
      }
      return;
    }
    next();
  };

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || !currentTrack) return;
    audio.src = convertFileSrc(currentTrack.path);
    audio.volume = muted ? 0 : volume;
    if (isPlaying) void audio.play();
    if (recordedRef.current !== currentTrack.id) {
      recordedRef.current = currentTrack.id;
      void api.recordPlay(currentTrack.id);
    }
  }, [currentTrack?.id]);

  useEffect(() => {
    const audio = audioRef.current;
    if (audio) audio.volume = muted ? 0 : volume;
  }, [volume, muted]);

  const playTracks = useCallback((tracks: Track[], startIndex = 0, shuf = shuffle) => {
    if (!tracks.length) return;
    setQueue(tracks);
    if (shuf) {
      const order = shuffleArray(tracks.map((_, i) => i));
      setShuffledOrder(order);
      setQueueIndex(order[startIndex] ?? order[0]);
    } else {
      setShuffledOrder([]);
      setQueueIndex(startIndex);
    }
    setIsPlaying(true);
    const audio = audioRef.current;
    if (audio) void audio.play();
  }, [shuffle]);

  const togglePlay = useCallback(() => {
    const audio = audioRef.current;
    if (!audio || !currentTrack) return;
    if (audio.paused) void audio.play();
    else audio.pause();
  }, [currentTrack]);

  const resolveIndex = useCallback(
    (delta: number) => {
      if (!queue.length) return queueIndex;
      if (shuffle && shuffledOrder.length) {
        const pos = shuffledOrder.indexOf(queueIndex);
        const nextPos = (pos + delta + shuffledOrder.length) % shuffledOrder.length;
        return shuffledOrder[nextPos];
      }
      return (queueIndex + delta + queue.length) % queue.length;
    },
    [queue, queueIndex, shuffle, shuffledOrder],
  );

  const next = useCallback(() => {
    if (!queue.length) return;
    if (repeat === "all" || shuffle) {
      setQueueIndex(resolveIndex(1));
      setIsPlaying(true);
    } else if (queueIndex < queue.length - 1) {
      setQueueIndex(queueIndex + 1);
      setIsPlaying(true);
    } else {
      setIsPlaying(false);
    }
  }, [queue, queueIndex, repeat, shuffle, resolveIndex]);

  const prev = useCallback(() => {
    const audio = audioRef.current;
    if (audio && audio.currentTime > 3) {
      audio.currentTime = 0;
      return;
    }
    if (!queue.length) return;
    setQueueIndex(resolveIndex(-1));
    setIsPlaying(true);
  }, [queue, resolveIndex]);

  const seek = useCallback((time: number) => {
    const audio = audioRef.current;
    if (audio) audio.currentTime = time;
    setPosition(time);
  }, []);

  const cycleRepeat = useCallback(() => {
    setRepeat((r) => (r === "off" ? "all" : r === "all" ? "one" : "off"));
  }, []);

  const toggleShuffle = useCallback(() => {
    setShuffle((s) => {
      const next = !s;
      if (next && queue.length) {
        setShuffledOrder(shuffleArray(queue.map((_, i) => i)));
      }
      return next;
    });
  }, [queue]);

  return {
    currentTrack,
    queue,
    isPlaying,
    shuffle,
    repeat,
    volume,
    muted,
    position,
    duration,
    playTracks,
    togglePlay,
    next,
    prev,
    seek,
    setVolume,
    setMuted,
    cycleRepeat,
    toggleShuffle,
    setQueue,
  };
}
