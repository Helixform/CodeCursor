import * as React from "react";
import {
    useState,
    useEffect,
    useLayoutEffect,
    useCallback,
    useRef,
    useMemo,
} from "react";
import { VSCodeButton, VSCodeTextArea, VSCodePanels, VSCodePanelTab, VSCodePanelView } from "@vscode/webview-ui-toolkit/react";

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
        msg.contents = msg.contents.replace(/\\n/g, "\n"); // 使用正则表达式和replace方法
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
        await chatService.confirmPrompt(prompt, "Freeform");
        setPrompt("");
    }, [prompt, setPrompt, setMessages]);


    const handleCustom = useCallback(async () => {
        const chatService = await getServiceManager().getService<IChatService>(
            CHAT_SERVICE_NAME
        );   
        const strPrompt = `你是一个中文助手，请用中文回答我所有问题。 Can you add tests for this code? ${prompt}`
        await chatService.confirmPrompt(strPrompt, "Custom");
        setPrompt("");            
    }, [prompt, setPrompt, setMessages]);

    const handleGenVarAction = useCallback(async () => {
        const chatService = await getServiceManager().getService<IChatService>(
            CHAT_SERVICE_NAME
        );   
        await chatService.confirmPrompt(prompt, "GenVar");
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
            
            <VSCodePanels>
                <VSCodePanelTab id="AI">AI对话</VSCodePanelTab>
                <VSCodePanelTab id="search">代码库文档库搜索</VSCodePanelTab>
                <VSCodePanelTab id="genVar">变量名</VSCodePanelTab>

                <VSCodePanelView id="AI">
                    <div className="chat-input-area chat-input-area-ai">
                        <VSCodeTextArea
                            style={{ width: "100%" }}
                            rows={3}
                            placeholder={`bout the ${
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
                            {`提问 (${confirmShortcut.label})`}
                        </VSCodeButton>
                    </div>
                    <div className="chat-icon-area">
                        <VSCodeButton className="chat-icon" title="check" onClick={handleCustom}>
                            <span className="codicon codicon-check"></span>
                        </VSCodeButton>
                        <VSCodeButton className="chat-icon" title="account" onClick={handleCustom}>
                            <span className="codicon codicon-account"></span>
                        </VSCodeButton>
                        <VSCodeButton className="chat-icon" title="activate-breakpoints" onClick={handleCustom}>
                            <span className="codicon codicon-activate-breakpoints"></span>
                        </VSCodeButton>
                        <VSCodeButton className="chat-icon" title="add" onClick={handleCustom}>
                            <span className="codicon codicon-add"></span>
                        </VSCodeButton>
                        <VSCodeButton className="chat-icon" title="archive" onClick={handleCustom}>
                            <span className="codicon codicon-archive"></span>
                        </VSCodeButton>
                        <VSCodeButton className="chat-icon" title="debug" onClick={handleCustom}>
                            <span className="codicon codicon-debug"></span>
                        </VSCodeButton>
                        <VSCodeButton className="chat-icon" title="color-mode" onClick={handleCustom}>
                            <span className="codicon codicon-color-mode"></span>
                        </VSCodeButton>
                        <VSCodeButton className="chat-icon" title="github-inverted" onClick={handleCustom}>
                            <span className="codicon codicon-github-inverted"></span>
                        </VSCodeButton>
                        <VSCodeButton className="chat-icon" title="heart" onClick={handleCustom}>
                            <span className="codicon codicon-heart"></span>
                        </VSCodeButton>
                    </div>
                </VSCodePanelView>

                <VSCodePanelView id="search">
                    <div className="chat-input-area">
                        <VSCodeTextArea
                            style={{ width: "100%" }}
                            rows={3}
                            placeholder={`云雀研发云文档库、代码库进行搜索...`}
                            disabled={!isReady}
                            value={prompt}
                            onInput={(e: React.ChangeEvent<HTMLTextAreaElement>) => {
                                setPrompt(e.target.value);
                            }}
                            onKeyDown={confirmShortcut.keyDownHandler}
                        />
                        <div style={{display:"flex", flexDirection:"row", justifyContent:"flex-end", gap:"20px" ,width:"100%"}}>
                            <VSCodeButton
                                disabled={!isReady || prompt.length === 0}
                                onClick={handleAskAction}
                            >
                                {'代码库搜索'}
                            </VSCodeButton>
                            <VSCodeButton
                                disabled={!isReady || prompt.length === 0}
                                onClick={handleAskAction}
                            >
                                {'文档库搜索'}
                            </VSCodeButton>                            
                        </div>
                    </div>
                </VSCodePanelView>     

                <VSCodePanelView id="genVar">
                <div className="chat-input-area">
                        <VSCodeTextArea
                            style={{ width: "100%" }}
                            rows={3}
                            placeholder={`请输入要生成的变量名提示语`}
                            disabled={!isReady}
                            value={prompt}
                            onInput={(e: React.ChangeEvent<HTMLTextAreaElement>) => {
                                setPrompt(e.target.value);
                            }}
                            onKeyDown={confirmShortcut.keyDownHandler}
                        />
                        <VSCodeButton
                            disabled={!isReady || prompt.length === 0}
                            onClick={handleGenVarAction}
                        >
                            {`生成变量名`}
                        </VSCodeButton>
                    </div>
                </VSCodePanelView>                                
            </VSCodePanels>
        </div>
    );
}
