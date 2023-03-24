import { GenerateSession } from "./generate";

interface GlobalState {
    activeSession: GenerateSession | null;
}

const globalState: GlobalState = {
    activeSession: null,
};

export function getGlobalState() {
    return globalState;
}
