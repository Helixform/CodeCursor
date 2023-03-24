export interface IService {
    get name(): string;
}

export interface IMessageReplyPort {
    sendReply(msg: unknown): void;
}

// Mapped type for extracting async methods in a service.
type _AsyncMethodKeys<T> = {
    [K in keyof T]: T[K] extends (...args: any[]) => infer R
        ? R extends Promise<any>
            ? K
            : never
        : never;
}[keyof T];
export type ServiceStub<S extends IService> = {
    [K in _AsyncMethodKeys<S>]: S[K];
} & { readonly name: string };

export const MESSAGE_TYPE_REQUEST = "req";
export const MESSAGE_TYPE_RESPONSE = "resp";

interface ServiceDescriptor {
    get host(): ServiceHost<any> | null;
    get serviceName(): string;
    get methodNames(): string[];
}

export class ServiceManager {
    #localServices = new Map<string, ServiceDescriptor>();

    registerService<S extends IService>(service: S) {
        const name = service.name;
        if (this.#localServices.has(name)) {
            throw Error(`Service named "${name} has already been registered"`);
        }

        const servicePrototype = service.constructor.prototype;
        const candidateServiceNames =
            Object.getOwnPropertyNames(servicePrototype);

        this.#localServices.set(name, {
            host: new ServiceHost(service),
            serviceName: name,
            methodNames: candidateServiceNames.filter((n) => {
                return servicePrototype[n] instanceof Function;
            }),
        });
    }

    async getService<S extends IService>(
        name: string
    ): Promise<ServiceStub<S>> {
        const localService = this.#localServices.get(name);
        if (localService) {
            return localService.host!.service as any as ServiceStub<S>;
        }

        // Cannot found a local service, try retrieving the remote service.
        const reply = await this.sendOutgoingMessageAndWaitForReply({
            type: "getService",
            serviceName: name,
        });

        try {
            // TODO: maybe we should check the type shape of `reply`, but it's fine
            // currently, because we only care about the these two field.
            const { error, methodNames: _methodNames } = reply as any;
            if (error) {
                throw new Error(error);
            }

            const methodNames: string[] = [];
            for (const methodName of _methodNames) {
                if (typeof methodName === "string") {
                    methodNames.push(methodName as string);
                }
            }

            // Return a proxy object.
            return new RemoteServiceProxy(this, {
                host: null,
                serviceName: name,
                methodNames,
            }) as any as ServiceStub<S>;
        } catch (e) {
            throw e;
        }
    }

    async handleIncomingMessage(msg: unknown): Promise<unknown> {
        const { type, serviceName } = msg as any;

        const that = this;
        function _getServiceDescriptor() {
            if (!serviceName || typeof serviceName !== "string") {
                return null;
            }
            const serviceDescriptor = that.#localServices.get(serviceName);
            if (!serviceDescriptor) {
                return null;
            }
            return serviceDescriptor;
        }

        if (type === "getService") {
            const serviceDescriptor = _getServiceDescriptor();
            if (!serviceDescriptor) {
                return {
                    error: "Invalid args or service not found",
                };
            }
            return {
                methodNames: serviceDescriptor.methodNames,
            };
        } else if (type === "invoke") {
            const { methodName, args } = msg as any;
            if (
                !methodName ||
                typeof methodName !== "string" ||
                !(args instanceof Array)
            ) {
                return {
                    error: "Invalid args",
                };
            }
            const serviceDescriptor = _getServiceDescriptor();
            if (!serviceDescriptor) {
                return {
                    error: "Invalid args or service not found",
                };
            }
            // This must be a local service.
            try {
                const result = await serviceDescriptor.host!.invokeMethod(
                    methodName,
                    args
                );
                return { result };
            } catch (e) {
                return {
                    error: `${e}`,
                };
            }
        }

        return {
            error: "Unknown message type",
        };
    }

    sendOutgoingMessageAndWaitForReply(msg: unknown): Promise<unknown> {
        const replyPort: IMessageReplyPort = {
            sendReply: null as any, // Late-init.
        };
        return new Promise((resolve) => {
            replyPort.sendReply = (msg) => {
                resolve(msg);
            };
            this.sendOutgoingMessage(msg, replyPort);
        });
    }

    protected sendOutgoingMessage(
        _msg: unknown,
        _replyPort: IMessageReplyPort
    ) {
        throw new Error("Not implemented");
    }
}

class ServiceHost<S extends IService> {
    #service: S;

    constructor(service: S) {
        this.#service = service;
    }

    get service(): S {
        return this.#service;
    }

    async invokeMethod(methodName: string, args: unknown[]): Promise<unknown> {
        const service = this.#service as any;
        const method = service[methodName];
        if (!(method instanceof Function)) {
            throw new Error(`"${methodName}" is not a function`);
        }
        // Every callable method is an async function.
        const result = await (method as Function).apply(service, args);
        return result;
    }
}

class RemoteServiceProxy {
    #serviceManager: ServiceManager;
    #serviceName: string;

    constructor(serviceManager: ServiceManager, descriptor: ServiceDescriptor) {
        this.#serviceManager = serviceManager;
        this.#serviceName = descriptor.serviceName;

        // Create stub methods.
        for (const methodName of descriptor.methodNames) {
            this.#addStubMethod(methodName);
        }
    }

    get name(): string {
        return this.#serviceName;
    }

    #addStubMethod(methodName: string) {
        const that = this;
        Object.defineProperty(this, methodName, {
            get() {
                return async function () {
                    const args = Array.from(arguments);
                    return await that.#invokeRemoteMethod(methodName, args);
                };
            },
            configurable: false,
            enumerable: true,
        });
    }

    async #invokeRemoteMethod(
        methodName: string,
        args: unknown[]
    ): Promise<unknown> {
        const reply =
            await this.#serviceManager.sendOutgoingMessageAndWaitForReply({
                type: "invoke",
                serviceName: this.#serviceName,
                methodName,
                args,
            });

        const { error, result } = reply as any;
        if (error) {
            // TODO: use a special `Error` type.
            throw new Error(error);
        }

        return result;
    }
}
