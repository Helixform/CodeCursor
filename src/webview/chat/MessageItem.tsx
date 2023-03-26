import * as React from "react";
import ReactMarkdown from "react-markdown";

import { MessageItemModel } from "../../common/chatService/model";
import { MessageCodeBlock } from "./MessageCodeBlock";

export interface MessageItemProps {
    model: MessageItemModel;
}

export function MessageItem(props: MessageItemProps) {
    const { model } = props;
    const { contents, isReply, isFinished } = model;
    return (
        <div className={`chat-msg ${isReply ? "reply" : ""}`}>
            <div className="chat-msg-contents">
                <MessageTextView
                    contents={
                        contents + (isReply && !isFinished ? "\u{258A}" : "")
                    }
                />
            </div>
        </div>
    );
}

interface MessageTextViewProps {
    contents: string;
}

function MessageTextView(props: MessageTextViewProps) {
    const { contents } = props;
    return (
        <ReactMarkdown
            components={{
                pre({ children, ...props }) {
                    if (children.length !== 1) {
                        // Not code block.
                        return <pre {...props}>{children}</pre>;
                    }
                    const child = children[0] as React.ReactElement;
                    const codeContents = child.props.children[0];
                    const codeClassName = child.props.className;
                    const languageMatch =
                        /language-(\w+)/.exec(codeClassName || "") || [];
                    return (
                        <MessageCodeBlock
                            contents={codeContents}
                            language={languageMatch[1] || ""}
                        />
                    );
                },
            }}
        >
            {contents}
        </ReactMarkdown>
    );
}
