import { Idea } from "./Idea";

// Manages core state and functionality of the program.
export class DaVinciBot
{
    private _rootIdea: Idea = new Idea();

    private _processes: { [name: string]: BotProcess } = {};
    private _currentProcess: string = '';

    private _status: BotStatus = BotStatus.Idle;

    get currentProcess(): string {
        return this._currentProcess;
    }

    get processes(): { [name: string]: BotProcess } {
        return this._processes;
    }

    get status(): BotStatus {
        return this._status;
    }

    addProcess(name: string, process: BotProcess) {
        this._processes[name] = process;
    }

    startProcess(process: string) {
        if (this._status !== BotStatus.Idle) {
            throw new Error(`Tried to switch BotProcecss without first finishing process ${this._currentProcess}`);
        }

        this._currentProcess = process;
        this._status = this._processes[process].start(this._rootIdea);
    }

    getOutput(): string {
        let outputTuple = this._processes[this._currentProcess].getOutput(this._rootIdea);

        this._status = outputTuple[1];
        return outputTuple[0];
    }

    handleInput(input: string) {
        this._status = this._processes[this._currentProcess].handleInput(input, this._rootIdea);
    }

    finishCurrentProcess() {
        this._processes[this._currentProcess].finish(this._rootIdea);
        this._status = BotStatus.Idle;
    }
}

export enum BotStatus
{
    HasOutput,
    NeedsInput,
    Idle
}

export class BotProcess
{
    description(): string { return ''; }

    start(rootIdea: Idea): BotStatus { throw new Error(`Custom BotProcess must define a start method: ${typeof this}`); }

    getOutput(rootIdea: Idea): [string, BotStatus] { return ['', BotStatus.Idle]; }
    handleInput(input: string, rootIdea: Idea): BotStatus { return BotStatus.Idle; }

    finish(rootIdea: Idea): void { }
}
