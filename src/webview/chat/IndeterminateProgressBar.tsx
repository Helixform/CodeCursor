import * as React from "react";
import { useState, useEffect, useRef } from "react";

function useAnimationFrame(callback: (time: number) => void, deps: any[]) {
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
    }, deps); // Make sure the effect runs only once
}

function cubicBezier(t: number) {
    return 3 * (1 - t) ** 2 * t * 0.1 + 3 * (1 - t) * t ** 2 * 0.9 + t ** 3;
}

export function IndeterminateProgressBar() {
    const indicator = useRef<HTMLDivElement>(null);

    const durationRef = useRef<number>(0);
    const [fastMode, setFastMode] = useState(false);

    const lengthRatio = (fastMode ? 0.5 : 0.2) * 100;
    const slowDuration = 1.5 * 1e3;
    const fastDuration = 0.8 * 1e3;

    useAnimationFrame(
        (deltaTime) => {
            if (!indicator.current) {
                return;
            }

            const progress =
                durationRef.current / (fastMode ? fastDuration : slowDuration);

            const newLeft =
                (100 + lengthRatio) * cubicBezier(progress) - lengthRatio;
            if (progress > 1) {
                indicator.current.style.left = `-${lengthRatio}%`;
                setFastMode(!fastMode);
                durationRef.current = 0;
            } else {
                indicator.current.style.left = `${newLeft}%`;
                durationRef.current += deltaTime;
            }
        },
        [fastMode, setFastMode]
    );

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
                    height: "100%",
                    background: "var(--vscode-progressBar-background)",
                    top: 0,
                    left: `-${lengthRatio}%`,
                }}
            ></div>
        </div>
    );
}
