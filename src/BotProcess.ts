import {DaVinciBot} from "./DaVinciBot";
import { Idea } from "./Idea";

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

export enum BotStatus
{
    HasOutput,
    NeedsInput,
    Idle
}
