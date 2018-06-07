import {Idea} from "./Idea";
import {BotState, BotStatus} from "./DaVinciBot";

export class AddIdeaState implements BotState
{
    start(): BotStatus {
        return BotStatus.HasOutput;
    }

    getOutput(): [string, BotStatus] {
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
