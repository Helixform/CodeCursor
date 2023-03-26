import * as React from "react";
import { useEffect, useRef } from "react";

function useAnimationFrame(callback: (time: number) => void) {
    const requestRef = useRef<number>();
    const previousTimeRef = useRef<number>();

    const animate = (time: number) => {
        if (previousTimeRef.current !== undefined) {
            const deltaTime = time - previousTimeRef.current;
            callback(deltaTime);
        }
        previousTimeRef.current = time;
        requestRef.current = requestAnimationFrame(animate);
    };

    useEffect(() => {
        requestRef.current = requestAnimationFrame(animate);
        return () => cancelAnimationFrame(requestRef.current!);
    }, []); // Make sure the effect runs only once
}

export type IndeterminateProgressBarProps = {
    lengthRatio?: number;
    slowDuration?: number;
    fastDuration?: number;
};

function cubicBezier(t: number) {
    return 3 * (1 - t) ** 2 * t * 0.1 + 3 * (1 - t) * t ** 2 * 0.9 + t ** 3;
}

export function IndeterminateProgressBar(props: IndeterminateProgressBarProps) {
    const indicator = useRef<HTMLDivElement>(null);

    const durationRef = useRef<number>(0);
    const fastModeRef = useRef<boolean>(false);

    const lengthRatio = (props.lengthRatio ?? 0.3) * 100;
    const slowDuration = props.slowDuration ?? 1.5 * 1e3;
    const fastDuration = props.fastDuration ?? 0.8 * 1e3;

    useAnimationFrame((deltaTime) => {
        if (!indicator.current) {
            return;
        }

        const fastMode = fastModeRef.current;
        const progress =
            durationRef.current / (fastMode ? fastDuration : slowDuration);

        const newLeft =
            (100 + lengthRatio) * cubicBezier(progress) - lengthRatio;
        if (progress > 1) {
            indicator.current.style.left = `-${lengthRatio}%`;
            fastModeRef.current = !fastMode;
            durationRef.current = 0;
        } else {
            indicator.current.style.left = `${newLeft}%`;
            durationRef.current += deltaTime;
        }
    });

    return (
        <div
            style={{
                flex: 1,
                width: "100%",
                height: "2px",
                position: "relative",
            }}
        >
            <div
                ref={indicator}
                style={{
                    position: "absolute",
                    width: `${lengthRatio}%`,
                    height: "100vh",
                    background: "lightskyblue",
                    top: 0,
                    left: `-${lengthRatio}%`,
                }}
            ></div>
        </div>
    );
}
