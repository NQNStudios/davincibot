import { Idea } from "./Idea";

// Manages core state and functionality of the program.
export class DaVinciBot
{
    private _rootIdea: Idea = new Idea();

    private _states: { [name: string]: BotState } = {};
    private _currentState: string = '';

    private _status: BotStatus = BotStatus.Idle;

    get currentState(): string {
        return this._currentState;
    }

    get status(): BotStatus {
        return this._status;
    }

    addState(name: string, state: BotState) {
        this._states[name] = state;
    }

    startState(state: string) {
        if (this._status !== BotStatus.Idle) {
            throw new Error(`Tried to switch BotState without first finishing state ${this._currentState}`);
        }

        this._currentState = state;
        this._status = this._states[state].start();
    }

    getOutput(): string {
        let outputTuple = this._states[this._currentState].getOutput();

        this._status = outputTuple[1];
        return outputTuple[0];
    }

    handleInput(input: string) {
        this._status = this._states[this._currentState].handleInput(input, this._rootIdea);
    }

    finishCurrentState() {
        this._states[this._currentState].finish(this._rootIdea);
        this._status = BotStatus.Idle;
    }
}

export enum BotStatus
{
    HasOutput,
    NeedsInput,
    Idle
}

export interface BotState
{
    start(): BotStatus;

    getOutput(): [string, BotStatus];
    handleInput(input: string, rootIdea: Idea): BotStatus;

    finish(rootIdea: Idea): void;
}
