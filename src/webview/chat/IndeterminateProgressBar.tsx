import * as React from "react";
import { useEffect, useRef } from "react";

function useAnimationFrame(callback: (time: number) => void, deps?: any[]) {
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
    return 3 * (1 - t) * t ** 2 * 0.7 + t ** 3;
}

export function IndeterminateProgressBar() {
    const indicatorRef = useRef<HTMLDivElement>(null);

    const shortLength = 20;
    const longLength = 50;
    const lengthRatio = 100 + shortLength + longLength;

    const duration = 2.1 * 1e3;
    const elapsedRef = useRef(0);

    useAnimationFrame((deltaTime) => {
        if (!indicatorRef.current) {
            return;
        }

        const progress = cubicBezier(elapsedRef.current / duration);

        const newLeft = (100 + lengthRatio) * progress - lengthRatio;
        if (progress > 1) {
            indicatorRef.current.style.left = `-${lengthRatio}%`;
            elapsedRef.current = 0;
        } else {
            indicatorRef.current.style.left = `${newLeft}%`;
            elapsedRef.current += deltaTime;
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
                ref={indicatorRef}
                style={{
                    position: "absolute",
                    width: `${lengthRatio}%`,
                    height: "100%",
                    top: 0,
                    left: `-${lengthRatio}%`,
                }}
            >
                <div
                    style={{
                        position: "absolute",
                        background: "var(--vscode-progressBar-background)",
                        width: `${(longLength / lengthRatio) * 100}%`,
                        height: "100%",
                    }}
                />
                <div
                    style={{
                        position: "absolute",
                        background: "var(--vscode-progressBar-background)",
                        width: `${(shortLength / lengthRatio) * 100}%`,
                        height: "100%",
                        right: 0,
                    }}
                />
            </div>
        </div>
    );
}
