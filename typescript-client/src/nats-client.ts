import { connect, NatsConnection, RequestOptions } from 'nats';

export interface ClientConfig {
  servers: string | string[];
  maxReconnectAttempts?: number;
  reconnectTimeWait?: number;
  token?: string;
  user?: string;
  pass?: string;
  verbose?: boolean;
  debug?: boolean;
}

export class NatsClient {
  private connection: NatsConnection | null = null;
  
  constructor(private config: ClientConfig) {}

  async connect(): Promise<void> {
    if (this.connection) {
      throw new Error('Already connected');
    }

    this.connection = await connect({
      servers: Array.isArray(this.config.servers) 
        ? this.config.servers 
        : [this.config.servers],
      maxReconnectAttempts: this.config.maxReconnectAttempts ?? -1,
      reconnectTimeWait: this.config.reconnectTimeWait ?? 2000,
      token: this.config.token,
      user: this.config.user,
      pass: this.config.pass,
      verbose: this.config.verbose,
      debug: this.config.debug,
    });

    console.log(`Connected to NATS server(s): ${this.config.servers}`);
  }

  async disconnect(): Promise<void> {
    if (!this.connection) {
      return;
    }

    await this.connection.drain();
    this.connection = null;
    console.log('Disconnected from NATS');
  }

  async request<TRequest, TResponse>(
    subject: string, 
    data: TRequest,
    encode: (message: TRequest) => Uint8Array,
    decode: (data: Uint8Array) => TResponse,
    options?: RequestOptions
  ): Promise<TResponse> {
    if (!this.connection) {
      throw new Error('Not connected to NATS');
    }

    const encoded = encode(data);
    const response = await this.connection.request(subject, encoded, options);
    return decode(response.data);
  }

  async publish<T>(
    subject: string,
    data: T,
    encode: (message: T) => Uint8Array
  ): Promise<void> {
    if (!this.connection) {
      throw new Error('Not connected to NATS');
    }

    const encoded = encode(data);
    this.connection.publish(subject, encoded);
  }

  async subscribe<T>(
    subject: string,
    decode: (data: Uint8Array) => T,
    handler: (message: T) => Promise<void> | void
  ): Promise<void> {
    if (!this.connection) {
      throw new Error('Not connected to NATS');
    }

    const subscription = this.connection.subscribe(subject);
    
    (async () => {
      for await (const msg of subscription) {
        try {
          const decoded = decode(msg.data);
          await handler(decoded);
        } catch (error) {
          console.error(`Error processing message on ${subject}:`, error);
        }
      }
    })();
  }

  isConnected(): boolean {
    return this.connection !== null;
  }

  getConnection(): NatsConnection | null {
    return this.connection;
  }
}