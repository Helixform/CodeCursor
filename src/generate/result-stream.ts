export class ResultStream<T> {
    write = (value: T) => {
        console.log(value);
    };

    end = () => {
        console.log("finished");
    };
}
