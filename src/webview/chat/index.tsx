import * as React from "react";
import {
    useState,
    useEffect,
    useLayoutEffect,
    useCallback,
    useRef,
    useMemo,
} from "react";
import { VSCodeButton, VSCodeTextArea } from "@vscode/webview-ui-toolkit/react";

import "./style.css";
import { MessageItem } from "./MessageItem";
import { ChatViewServiceImpl } from "./chatViewServiceImpl";
import { getServiceManager } from "../../common/ipc/webview";
import { IChatService, CHAT_SERVICE_NAME } from "../../common/chatService";
import { MessageItemModel } from "../../common/chatService/model";

function messagesWithUpdatedBotMessage(
    msgs: MessageItemModel[],
    updatedMsg: MessageItemModel
): MessageItemModel[] {
    return msgs.map((msg) => {
        if (updatedMsg.id === msg.id) {
            return updatedMsg;
        }
        return msg;
    });
}

type UseConfirmShortcut = {
    label: string;
    keyDownHandler: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
};
function useConfirmShortcut(handler: () => void): UseConfirmShortcut {
    const isMac = useMemo(() => {
        const userAgentData = (window.navigator as any).userAgentData;
        if (userAgentData) {
            return userAgentData.platform === "macOS";
        }
        return window.navigator.platform === "MacIntel";
    }, []);

    return {
        label: isMac ? "⌘⏎" : "Ctrl+Enter",
        keyDownHandler: useCallback(
            (e) => {
                if (e.key !== "Enter") {
                    return;
                }
                const expected = isMac ? e.metaKey : e.ctrlKey;
                const unexpected = isMac ? e.ctrlKey : e.metaKey;
                if (!expected || e.altKey || e.shiftKey || unexpected) {
                    return;
                }
                handler();
            },
            [isMac, handler]
        ),
    };
}

const AUTO_SCROLL_FLAG_NONE = 0;
const AUTO_SCROLL_FLAG_FORCED = 1;
const AUTO_SCROLL_FLAG_AUTOMATIC = 2;

export function ChatPage() {
    const [messages, setMessages] = useState([] as MessageItemModel[]);
    const [hasSelection, setHasSelection] = useState(false);
    const [isReady, setIsReady] = useState(false);
    const [prompt, setPrompt] = useState("");
    const [autoScrollFlag, setAutoScrollFlag] = useState(AUTO_SCROLL_FLAG_NONE);
    const chatListRef = useRef<HTMLDivElement>(null);

    // Dependent on `setMessages`, which will never change.
    const addMessageAction = useCallback((msg: MessageItemModel) => {
        setMessages((prev) => {
            return [...prev, msg];
        });
        setAutoScrollFlag(AUTO_SCROLL_FLAG_FORCED);
    }, []);
    const updateMessageAction = useCallback((msg: MessageItemModel) => {
        setMessages((prev) => {
            return messagesWithUpdatedBotMessage(prev, msg);
        });
        setAutoScrollFlag(AUTO_SCROLL_FLAG_AUTOMATIC);
    }, []);
    const clearMessageAction = useCallback(() => {
        setMessages([]);
    }, []);

    const handleAskAction = useCallback(async () => {
        const chatService = await getServiceManager().getService<IChatService>(
            CHAT_SERVICE_NAME
        );
        await chatService.confirmPrompt(prompt);
        setPrompt("");
    }, [prompt, setPrompt, setMessages]);

    const confirmShortcut = useConfirmShortcut(handleAskAction);

    useLayoutEffect(() => {
        if (!autoScrollFlag) {
            return;
        }
        const chatListEl = chatListRef.current;
        if (!chatListEl) {
            return;
        }

        setAutoScrollFlag(AUTO_SCROLL_FLAG_NONE);

        const targetScrollTop =
            chatListEl.scrollHeight - chatListEl.clientHeight;
        // TODO: implement `AUTO_SCROLL_FLAG_AUTOMATIC` flag.
        chatListEl.scrollTop = targetScrollTop;
    }, [messages, autoScrollFlag, setAutoScrollFlag, chatListRef]);

    useEffect(() => {
        const serviceManager = getServiceManager();

        const viewServiceImpl = new ChatViewServiceImpl();
        viewServiceImpl.setIsReadyAction = setIsReady;
        viewServiceImpl.setHasSelectionAction = setHasSelection;
        viewServiceImpl.addMessageAction = addMessageAction;
        viewServiceImpl.updateMessageAction = updateMessageAction;
        viewServiceImpl.clearMessageAction = clearMessageAction;
        serviceManager.registerService(viewServiceImpl);

        serviceManager
            .getService<IChatService>(CHAT_SERVICE_NAME)
            .then((chatService) => {
                chatService.syncState();
            });
    }, []);

    return (
        <div className="chat-root">
            <div ref={chatListRef} className="chat-list">
                {messages.map((m) => {
                    return <MessageItem key={m.id} model={m} />;
                })}
            </div>
            <div className="chat-input-area">
                <VSCodeTextArea
                    style={{ width: "100%" }}
                    rows={3}
                    placeholder={`Talk about the ${
                        hasSelection ? "selected contents" : "whole document"
                    }...`}
                    disabled={!isReady}
                    value={prompt}
                    onInput={(e: React.ChangeEvent<HTMLTextAreaElement>) => {
                        setPrompt(e.target.value);
                    }}
                    onKeyDown={confirmShortcut.keyDownHandler}
                />
                <VSCodeButton
                    disabled={!isReady || prompt.length === 0}
                    onClick={handleAskAction}
                >
                    {`Ask (${confirmShortcut.label})`}
                </VSCodeButton>
            </div>
        </div>
    );
}
