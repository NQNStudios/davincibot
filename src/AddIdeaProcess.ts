import {Idea} from "./Idea";
import {BotProcess, BotStatus} from "./DaVinciBot";

export class AddIdeaProcess extends BotProcess
{
    description(): string { return 'Add ideas to the idea pool.'; }

    start(): BotStatus {
        return BotStatus.HasOutput;
    }

    getOutput(): [string, BotStatus] {
        return [`Enter as many ideas as you want, followed by ENTER. To stop entering ideas, type 'quit'`, BotStatus.NeedsInput];
    }

    handleInput(input: string): BotStatus {
        if (input === 'quit') {
            return BotStatus.Idle;
        }

        this.rootIdea.addChild(input);
        return BotStatus.NeedsInput;
    }

    finish(): void {
        // TODO don't always do this
        console.log(this.rootIdea.children);
    }
};
