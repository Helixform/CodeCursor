import * as React from "react";
import { useCallback } from "react";
import { VSCodeButton } from "@vscode/webview-ui-toolkit/react";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";

import { getServiceManager } from "../../common/ipc/webview";
import { IChatService, CHAT_SERVICE_NAME } from "../../common/chatService";

export interface MessageCodeBlockProps {
    contents: string;
    language: string;
}

export function MessageCodeBlock(props: MessageCodeBlockProps) {
    const { contents, language } = props;
    const handleCopyAction = useCallback(() => {
        navigator.clipboard.writeText(contents);
    }, [contents]);
    const handleInsertCodeSnippetAction = useCallback(async () => {
        const chatService = await getServiceManager().getService<IChatService>(
            CHAT_SERVICE_NAME
        );
        await chatService.insertCodeSnippet(contents);
    }, [contents]);

    return (
        <>
            <div className="chat-msg-block-toolbar">
                <VSCodeButton
                    appearance="icon"
                    ariaLabel="Copy"
                    title="Copy"
                    onClick={handleCopyAction}
                >
                    <span className="codicon codicon-copy"></span>
                </VSCodeButton>
                <VSCodeButton
                    appearance="icon"
                    ariaLabel="Insert or Replace"
                    title="Insert or Replace"
                    onClick={handleInsertCodeSnippetAction}
                >
                    <span className="codicon codicon-insert"></span>
                </VSCodeButton>
            </div>
            <SyntaxHighlighter
                useInlineStyles={false}
                codeTagProps={{ style: {} }}
                language={language}
            >
                {contents}
            </SyntaxHighlighter>
        </>
    );
}
