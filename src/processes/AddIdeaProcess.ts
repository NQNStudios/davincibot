import {Idea} from "../Idea";
import {BotProcess, BotStatus} from "../DaVinciBot";

export class AddIdeaProcess extends BotProcess
{
    start(): void {
        this.status = BotStatus.HasOutput;
    }

    getOutput(): string {
        this.status = BotStatus.NeedsInput;
        return `Enter as many ideas as you want, followed by ENTER. To stop entering ideas, type 'quit'`;
    }

    handleInput(input: string): void {
        if (input === 'quit') {
            this.status = BotStatus.Idle;
        }
        else {
            this.rootIdea.addChild(input);
            this.status = BotStatus.NeedsInput;
        }
    }

    finish(): void {
        // TODO don't always do this
        console.log(this.rootIdea.children);
    }
}
