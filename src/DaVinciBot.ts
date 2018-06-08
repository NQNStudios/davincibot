import { Idea } from "./Idea";

// Manages a call stack of BotProcesses
export class DaVinciBot
{
    private _processes: Array<BotProcess> = [];

    get currentProcess(): BotProcess {
        return this._processes[this._processes.length-1];
    }

    get status(): BotStatus {
        if (this._processes.length == 0) {
            return BotStatus.Idle;
        }

        return this.currentProcess.status;
    }

    startProcess(process: BotProcess) {
        this._processes.push(process);
        process.start();
    }

    getOutput(): string {
        let output = this.currentProcess.getOutput();
        this.finishProcessIfIdle();
        return output;
    }

    handleInput(input: string) {
        this.currentProcess.handleInput(input);
        this.finishProcessIfIdle();
    }

    finishCurrentProcess() {
        this.currentProcess.finish();
        this._processes.pop();
    }

    finishProcessIfIdle() {
        if (this.status === BotStatus.Idle) {
            this.finishCurrentProcess();
        }
    }
}

export enum BotStatus
{
    HasOutput,
    NeedsInput,
    Idle
}

// TODO make this its own file
export class BotProcess
{
    bot: DaVinciBot;
    rootIdea: Idea;
    status: BotStatus = BotStatus.Idle;

    constructor(bot: DaVinciBot, rootIdea: Idea) {
        this.bot = bot;
        this.rootIdea = rootIdea;
    }

    start(): void { }
    getOutput(): string { return ''; }
    handleInput(input: string): void { }
    finish(): void { }
}
