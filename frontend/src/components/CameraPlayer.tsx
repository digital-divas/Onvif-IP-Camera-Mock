import { useEffect, useRef } from "react";
import Hls from "hls.js";

type Props = {
    src: string;
};

export default function CameraPlayer({ src }: Props) {
    const videoRef = useRef<HTMLVideoElement | null>(null);

    useEffect(() => {
        const video = videoRef.current;

        if (!video) return;

        let hls: Hls | null = null;

        if (video.canPlayType("application/vnd.apple.mpegurl")) {
            // Safari
            video.src = src;
        } else if (Hls.isSupported()) {
            hls = new Hls({
                enableWorker: true,
                lowLatencyMode: true,
            });

            hls.loadSource(src);
            hls.attachMedia(video);
        }

        return () => {
            if (hls) {
                hls.destroy();
            }
        };
    }, [src]);

    return (
        <div style={{ flex: 1, minHeight: 820, maxHeight: 820 }}>
            <video
                ref={videoRef}
                controls={false}
                autoPlay
                muted
                style={{ width: "100%", height: "100%", objectFit: "contain", }}
            />
        </div>
    );
}