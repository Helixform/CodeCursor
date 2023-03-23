# Cursor for Visual Studio Code

**Use Cursor right in the editor you love!**

First of all, We would like to thank **Cursor Team** for their brilliant works. Please give their [app](https://www.cursor.so) a try!

## What's Cursor? And Why This Extension?

Cursor is an AI code editor based on GPT-4. You can write, edit and chat about your code with it. At this time, Cursor is only provided as a dedicated app, and the team currently has no plans to develop extensions for other editors or IDEs.

We believe there are more developers actively use Visual Studio Code as their main tool for serious works. And this is why we built **Code Cursor**. It's not going to replace the Cursor app, but it provides another way to use Cursor.

## Getting Started

You don't need to configure anything before starting using it. Just open a document and type `Code Cursor` in Command Palette. You will see the command below:

![Command Palette](./artworks/command-palette.png)

Type your prompt and the code generation will just begin. To edit some existing code, you can also select something before perform this command, when accepting the change, the selected code will be replaced with the generated one.

While code generation is in progress, the following status bar item will be displayed:

![Generating](./artworks/generating.png)

Click on it to cancel the request.

Upon completion of code generation, the status bar item will change to:

![Completed](./artworks/completed.png)

Click on it to reopen the generated result at any time.

## Known Issues

- [ ]  Code generation may be interrupted unexpectedly, this is still being investigated. Generally, a retry will fix it.
- [x]  When users modified the document before accepting a change, the replacing range is incorrect.

To track all issues / file a new issue please go to the Github repo.

## Security Consideration

The extension **DOES NOT** collect your code, environment data, or any information that could be used to track you. Additionally, we ensure that the Cursor server will not receive those data either. Only the document you perform code generation against will be uploaded to the Cursor server, and they are responsible for preventing any leaks of your code.

## Contributing

To develop the extension, clone the repository and open it in Visual Studio Code. There are two launch targets: "Run Extension" and "Run Extension (Without Rust)". if you only want to debug or work on the UI parts, then you can select "Run Extension (Without Rust)" for faster build speed.

You are welcome to open Pull Requests at any time. But it's still better to start a discussion before making some epic changes.

## License

MIT
