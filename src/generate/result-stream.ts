export interface ResultStream<T> {
    write(value: T): void;
    end(): void;
}
