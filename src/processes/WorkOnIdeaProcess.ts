import {Idea} from "../Idea";
import {BotProcess, BotStatus} from "../DaVinciBot";
import {PlanIdeaProcess} from "./PlanIdeaProcess";

export class WorkOnIdeaProcess extends BotProcess
{
    start(): void {
        this.status = BotStatus.HasOutput;
    }

    getOutput(): string {
        this.status = BotStatus.NeedsInput;
        return `Working on idea ${this.rootIdea.name}:
            1. Mark this idea [done].
            2. Work on this idea's [parts]
            3. [Delete] this idea
            4. [Plan] this idea.
            5. [Clear] finished children
            6. [quit]
        `;
    }

    handleInput(input:string) {
        this.status = BotStatus.HasOutput;
        switch (input.toLowerCase()) {
            case '1':
            case 'done':
                this.rootIdea.progress = 1;
                this.status = BotStatus.Idle;
                break;
            case '2':
            case 'parts':
                for (let i = this.rootIdea.children.length - 1; i >= 0; --i) {
                    let child = this.rootIdea.children[i];
                    if (child.progress != 1) {
                        this.bot.startProcess(new WorkOnIdeaProcess(this.bot, child));
                    }
                }
                break;
            case '3':
            case 'delete':
                // TODO delete this idea from its root
                break;
            case '4':
            case 'plan':
                this.bot.startProcess(new PlanIdeaProcess(this.bot, this.rootIdea));
            case '5':
            case 'clear':
                // TODO clear out all children with progress of 1
                break;
            case '6':
            case 'quit':
                this.status = BotStatus.Idle;
                break;
        }
    }

}
