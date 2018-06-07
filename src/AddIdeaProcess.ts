import {Idea} from "./Idea";
import {BotProcess, BotStatus} from "./DaVinciBot";

export class AddIdeaProcess extends BotProcess
{
    description(): string { return 'Add ideas to the idea pool.'; }

    start(rootIdea: Idea): BotStatus {
        return BotStatus.HasOutput;
    }

    getOutput(rootIdea: Idea): [string, BotStatus] {
        return [`Enter as many ideas as you want, followed by ENTER. To stop entering ideas, type 'quit'`, BotStatus.NeedsInput];
    }

    handleInput(input: string, rootIdea: Idea): BotStatus {
        if (input === 'quit') {
            return BotStatus.Idle;
        }

        rootIdea.addChild(input);
        return BotStatus.NeedsInput;
    }

    finish(rootIdea: Idea): void {
        // TODO don't always do this
        console.log(rootIdea.children);
    }
};
